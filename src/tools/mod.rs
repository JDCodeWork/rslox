use std::{fs::File, io::Write};

use crate::{
    cli::Alert,
    errors::{Error, SystemError},
};

pub struct GenerateAst {}

impl GenerateAst {
    pub fn define_ast(output_dir: String, base_name: String, ast_types: Vec<String>) {
        let path = format!("{}/{}.rs", output_dir.to_lowercase(), base_name);

        let mut file = match File::create(&path) {
            Ok(f) => f,
            Err(..) => Error::from(SystemError::FiledToCreateFile(path)).report_and_exit(1),
        };

        let mut file_content = String::new();
        file_content.push_str("use super::token::Token;\n\n");

        file_content.push_str(format!("pub enum {} {{\n", &base_name).as_str());
        for _type in &ast_types {
            let struct_name = match _type.split(":").nth(0) {
                Some(sn) => sn.trim(),
                None => Alert::error(String::from("CLI | syntax error in metalanguage"))
                    .show_and_exit(1),
            };

            Self::define_enum(&mut file_content, struct_name);
        }
        file_content.push_str("}\n");

        for _type in ast_types {
            let struct_name = match _type.split(":").nth(0) {
                Some(sn) => sn.trim(),
                None => Alert::error(String::from("CLI | syntax error in metalanguage"))
                    .show_and_exit(1),
            };

            let fields = match _type.split(":").nth(1) {
                Some(f) => f.trim(),
                None => Alert::error(String::from("CLI | syntax error in metalanguage"))
                    .show_and_exit(1),
            };

            Self::define_struct(&mut file_content, &base_name, struct_name, &fields);
        }

        match file.write_all(file_content.as_bytes()) {
            Ok(..) => {}
            Err(..) => Error::from(SystemError::FiledToCreateFile(path)).report_and_exit(1),
        }
    }

    fn define_enum(file_content: &mut String, struct_name: &str) {
        file_content.push_str(format!("\t{}({}),\n", struct_name, struct_name).as_str());
    }

    fn define_struct(file_content: &mut String, base_name: &str, struct_name: &str, fields: &str) {
        file_content.push_str(format!("pub struct {} {{\n", struct_name).as_str());

        let fields_list = fields.split(",").map(|f| f.trim()).into_iter();

        for field in fields_list {
            let _type = match field.split(" ").nth(0) {
                Some(t) => t,
                None => Alert::error(String::from("CLI | syntax error in metalanguage"))
                    .show_and_exit(1),
            };

            let name = match field.split(" ").nth(1) {
                Some(n) => n,
                None => Alert::error(String::from("CLI | syntax error in metalanguage"))
                    .show_and_exit(1),
            };

            if _type == base_name {
                file_content.push_str(format!("\t{}: Box<{}>,\n", name, _type).as_str());
            } else {
                file_content.push_str(format!("\t{}: {},\n", name, _type).as_str());
            }
        }

        file_content.push_str("}\n");
    }
}
