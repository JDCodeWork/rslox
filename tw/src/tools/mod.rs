use std::{fs::File, io::Write};

use crate::{
    errors::{IoError, LoxError},
    lox::ast::{Expr, Stmt},
};

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(stmt: Stmt) -> String {
        stmt.print()
    }

    pub fn parenthesize(name: &str, exprs: Vec<Box<Expr>>) -> String {
        let mut builder = String::new();

        builder += format!("({name}").as_str();

        for expr in exprs {
            builder += " ";
            builder += expr.print().as_str()
        }
        builder += ")";

        builder
    }
}

pub struct AstGenerator {
    generated_code: String,
    base_enum_name: String,
    ast_types: Vec<AstType>,
}

struct AstType {
    type_name: String,
    type_fields: Vec<AstTypeField>,
}

struct AstTypeField {
    field_type: String,
    field_name: String,
}

impl AstGenerator {
    pub fn new(base_enum_name: String, raw_ast_types: Vec<String>) -> Self {
        let generated_code = String::new();
        let mut ast_types = Vec::new();

        for ast_type in raw_ast_types {
            let type_name = match ast_type.split(":").nth(0) {
                Some(sn) => sn.trim().to_string(),
                None => LoxError::Io(IoError::ASTSyntaxInvalid).report_and_exit(1),
            };

            let raw_type_fields = match ast_type.split(":").nth(1) {
                Some(f) => f.trim(),
                None => LoxError::Io(IoError::ASTSyntaxInvalid).report_and_exit(1),
            };

            let mut type_fields = Vec::new();
            for raw_type_field in raw_type_fields.split(",") {
                let mut field_data = raw_type_field.trim().split(" ");

                let field_type = if let Some(f_t) = field_data.next() {
                    f_t.to_string()
                } else {
                    LoxError::Io(IoError::ASTSyntaxInvalid).report_and_exit(1)
                };

                let field_name = if let Some(f_n) = field_data.next() {
                    f_n.to_string()
                } else {
                    LoxError::Io(IoError::ASTSyntaxInvalid).report_and_exit(1)
                };

                type_fields.push(AstTypeField {
                    field_type,
                    field_name,
                });
            }

            ast_types.push(AstType {
                type_name,
                type_fields,
            });
        }

        Self {
            generated_code,
            base_enum_name,
            ast_types,
        }
    }
}

impl AstGenerator {
    pub fn gen_(&mut self, output_dir: &String) {
        // imports
        self.add_content("use super::token::Token;\n\n");

        // Type with the highest hierarchy
        self.define_enums();

        // Structure with lower hierarchy
        self.define_structs();

        // Implementation of the associated function "Self::new(..args)"
        self.define_new_func();

        // Save the file with generated code
        self.save(output_dir);
    }

    fn add_content<T: Into<String>>(&mut self, content: T) {
        self.generated_code.push_str(&content.into());
    }

    fn define_enums(&mut self) {
        self.add_content(format!("pub enum {} {{\n", &self.base_enum_name));

        let mut temp_content = String::new();
        for ast_type in &self.ast_types {
            let AstType { type_name, .. } = ast_type;

            temp_content.push_str(format!("\t{}({}),\n", type_name, type_name).as_str());
        }

        self.add_content(temp_content);
        self.add_content("}\n\n");
    }

    fn define_structs(&mut self) {
        let mut temp_content = String::new();
        for ast_type in &self.ast_types {
            let AstType {
                type_name,
                type_fields,
            } = ast_type;

            temp_content.push_str(format!("pub struct {} {{\n", type_name).as_str());

            for type_field in type_fields {
                let AstTypeField {
                    field_type,
                    field_name,
                } = type_field;

                if *field_type == self.base_enum_name {
                    temp_content
                        .push_str(format!("\t{}: Box<{}>,\n", field_name, field_type).as_str());
                } else {
                    temp_content.push_str(format!("\t{}: {},\n", field_name, field_type).as_str());
                }
            }

            temp_content.push_str("}\n\n");
        }

        self.add_content(temp_content);
    }

    fn define_new_func(&mut self) {
        let mut temp_content = String::new();

        for ast_type in &self.ast_types {
            let AstType {
                type_name,
                type_fields,
            } = ast_type;

            temp_content.push_str(format!("impl {} {{\n", type_name).as_str());
            temp_content.push_str("\tpub fn new(");

            for type_field in type_fields {
                let AstTypeField {
                    field_type,
                    field_name,
                } = type_field;

                temp_content.push_str(format!("{}: {}, ", field_name, field_type).as_str());
            }
            temp_content.truncate(temp_content.len() - 2);
            temp_content.push_str(") -> Self {\n");

            temp_content.push_str("\t\tSelf {\n");
            for type_field in type_fields {
                let AstTypeField {
                    field_type,
                    field_name,
                } = type_field;

                if *field_type == self.base_enum_name {
                    temp_content.push_str(
                        format!("\t\t\t{}: Box::new({}),\n", field_name, field_name).as_str(),
                    );
                } else {
                    temp_content.push_str(format!("\t\t\t{},\n", field_name).as_str());
                }
            }

            temp_content.push_str("\t\t}\n");
            temp_content.push_str("\t}\n");
            temp_content.push_str("}\n\n");
        }

        self.add_content(temp_content);
    }

    fn save(&self, output_dir: &String) {
        let path = format!("{}/{}.rs", output_dir, self.base_enum_name.to_lowercase());

        let mut file = match File::create(&path) {
            Ok(f) => f,
            Err(..) => LoxError::Io(IoError::FailedToCreateFile(path)).report_and_exit(1),
        };

        if let Err(..) = file.write_all(self.generated_code.as_bytes()) {
            LoxError::Io(IoError::FailedToCreateFile(path)).report_and_exit(1);
        }
    }
}
