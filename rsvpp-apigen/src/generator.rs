use crate::{
    parser::{
        ApiAliase, ApiEnum, ApiEnumField, ApiEnumFlag, ApiField, ApiMessage, ApiService, ApiType,
        ApiUnion, ApiUnionField, JsonApi,
    },
    utils::Capitalize,
    Result,
};
use std::{collections::HashSet, iter::FromIterator};

lazy_static::lazy_static! {
    static ref RESERVED_SET: HashSet<&'static str> = HashSet::from_iter(vec![
        "as", "use", "extern crate", "break", "const", "continue", "crate", "else",
        "if", "if let", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
        "for", "let", "loop", "match", "mod", "move", "mut", "pub", "impl", "ref",
        "return", "Self", "self", "static", "struct", "super", "trait", "true", "type",
        "unsafe", "use", "where", "while", "abstract", "alignof", "become", "box", "do",
        "final", "macro", "offsetof", "override", "priv", "proc", "pure", "sizeof",
        "typeof", "unsized", "virtual", "yield"
    ]);
}

lazy_static::lazy_static! {
    static ref BASE_TYPE_SET: HashSet<&'static str> = HashSet::from_iter(vec![
        "bool", "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "f32", "f64",
    ]);
}

macro_rules! ensure_not_duplicate {
    ($name:expr, $set:expr) => {
        if $set.contains(&$name) {
            return Ok(Vec::new());
        } else {
            $set.insert($name);
        }
    };
}

pub struct Generator {
    in_dir: String,
    out_dir: String,
    error_header_file: String,
}

impl Generator {
    pub fn new<S: ToString>(out_dir: S, in_dir: S, error_header_file: S) -> Result<Self> {
        Ok(Self {
            in_dir: in_dir.to_string(),
            out_dir: out_dir.to_string(),
            error_header_file: error_header_file.to_string(),
        })
    }

    pub fn gen(&mut self) -> Result<()> {
        // Parse api
        let paths = glob::glob(&format!("{}/**/*.api.json", self.in_dir))?;
        let mut apis: Vec<JsonApi> = Vec::new();
        for path in paths {
            let path = path?;
            let file = path.to_str().unwrap_or("");
            println!("Parse api file: '{}'", file);
            apis.push(self.parse_single_file(file)?);
        }

        // Generate api
        for api in apis {
            println!("Generate rust code from api: '{}'", api.name);
            Self::gen_single_api(&api, &format!("{}/{}.rs", self.out_dir, api.name))?;
        }

        // Parse error
        println!("Parse error header file");
        let err_pairs = Self::parse_error_header_file(&self.error_header_file)?;

        // Generate error
        println!("Generate error map");
        Self::gen_error_map(err_pairs, &self.out_dir)?;

        Ok(())
    }

    fn parse_single_file(&mut self, file: &str) -> Result<JsonApi> {
        let buf = std::fs::read(file).map_err(|e| format!("Read file '{}' error: {}", file, e))?;
        let s = std::str::from_utf8(&buf)?;
        let api_name = &regex::Regex::new(r"(\w+)\.api\.json")?
            .captures(file)
            .ok_or("Invalid api filename")?[1];
        let api = JsonApi::parse(api_name.to_string(), s)
            .map_err(|e| format!("Decode file '{}', error: {}", file, e))?;

        Ok(api)
    }

    fn parse_error_header_file(file: &str) -> Result<Vec<(i32, String)>> {
        let buf = std::fs::read(file).map_err(|e| format!("Read '{}' error: {}", file, e))?;
        let content = std::str::from_utf8(&buf)?;
        let err_regex = regex::Regex::new(r#"_\s*\([A-Z_0-9a-z]+,\s*(\-?\d+),\s*"(.*)""#)?;
        let mut arr: Vec<(i32, String)> = Vec::new();
        for line in content.split("\n") {
            if let Some(cap) = err_regex.captures(line) {
                let err_code: i32 = cap[1].parse()?;
                let err_msg: String = cap[2].to_string();
                arr.push((err_code, err_msg));
            }
        }

        Ok(arr)
    }

    #[rustfmt::skip]
    fn gen_single_api(api: &JsonApi, file: &str) -> Result<()> {
        let mut lines: Vec<String> = Vec::new();
        let mut generated_type_set: HashSet<String> = HashSet::new();
        let has_retval_type_set: HashSet<String> = Self::get_has_retval_has_set(api)?;

        // Gen headers
        lines.push(format!("#![allow(unused)]\n"));
        #[rustfmt::skip]
        lines.push(format!("use rsvpp::pack::{{self, Pack, PackDefault, pack_union}};\n"));

        // Gen check error function
        lines.push(format!("fn check_error(retval: i32) -> rsvpp::Result<()> {{"));
        lines.push(format!("    if retval != 0 {{"));
        lines.push(format!("        if let Some(msg) = super::error_map::ERROR_MAP.get(&retval) {{"));
        lines.push(format!("            return Err(rsvpp::Error::vpp_api(format!(\"code: {{}}, msg: '{{}}'\", retval, msg)));"));
        lines.push(format!("        }} else {{"));
        lines.push(format!("            return Err(rsvpp::Error::vpp_api(format!(\"code: {{}}, msg: NULL\", retval)));"));
        lines.push(format!("        }}"));
        lines.push(format!("    }}"));
        lines.push(format!("    Ok(())"));
        lines.push(format!("}}\n"));

        // Gen alias
        for alias in &api.aliases {
            lines.extend(Self::gen_alias(alias, &mut generated_type_set)?);
        }

        // Gen types
        for ty in &api.types {
            lines.extend(Self::gen_type(ty, &mut generated_type_set)?);
        }

        // Gen messages
        for msg in &api.messages {
            lines.extend(Self::gen_message(msg, &mut generated_type_set)?);
        }

        // Gen unions
        for uni in &api.unions {
            lines.extend(Self::gen_union(uni, &mut generated_type_set)?);
        }

        // Gen enum_flags
        for enum_flag in &api.enum_flags {
            lines.extend(Self::gen_enum_flag(enum_flag, &mut generated_type_set)?);
        }

        // Gen enums
        for enu in &api.enums {
            lines.extend(Self::gen_enum(enu, &mut generated_type_set)?);
        }

        // Gen services
        lines.extend(Self::gen_services(
            &api.name,
            &api.services,
            &has_retval_type_set,
        )?);

        // Join code
        let code = lines.join("\n");

        // Write file
        std::fs::write(file, code)?;

        Ok(())
    }

    fn get_has_retval_has_set(api: &JsonApi) -> Result<HashSet<String>> {
        let mut set = HashSet::new();

        for ty in &api.types {
            if ty.fields.iter().any(|field| field.name == "retval") {
                set.insert(gen_struct_name(&ty.name));
            }
        }

        for msg in &api.messages {
            if msg.fields.iter().any(|field| field.name == "retval") {
                set.insert(gen_struct_name(&msg.name));
            }
        }

        Ok(set)
    }

    fn gen_error_map(errs: Vec<(i32, String)>, outdir: &str) -> Result<()> {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("use std::collections::HashMap;\n"));
        lines.push(format!("rsvpp::lazy_static::lazy_static!("));
        lines.push(format!(
            "    pub static ref ERROR_MAP: HashMap<i32, &'static str> = {{"
        ));
        lines.push(format!("        let mut m = HashMap::new();"));
        for pair in errs {
            lines.push(format!("        m.insert({}, \"{}\");", pair.0, pair.1));
        }
        lines.push(format!("        m"));
        lines.push(format!("    }};"));
        lines.push(format!(");\n"));

        let content = lines.join("\n");
        std::fs::write(format!("{}/error_map.rs", outdir), content)
            .map_err(|e| format!("Write error map file error: {}", e))?;

        Ok(())
    }

    fn gen_alias(
        alias: &ApiAliase,
        generated_type_set: &mut HashSet<String>,
    ) -> Result<Vec<String>> {
        ensure_not_duplicate!(gen_struct_name(&alias.name), generated_type_set);

        let mut lines: Vec<String> = Vec::new();

        let left = gen_struct_name(&alias.name);
        let right_type = gen_filed_type(&alias.ty);
        let right = if let Some(n) = alias.len {
            format!("[{}; {}]", right_type, n)
        } else {
            right_type
        };

        lines.push(format!("pub type {} = {};\n", left, right));

        Ok(lines)
    }

    fn gen_type(ty: &ApiType, generated_type_set: &mut HashSet<String>) -> Result<Vec<String>> {
        ensure_not_duplicate!(gen_struct_name(&ty.name), generated_type_set);

        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("#[derive(Pack, Debug, PackDefault)]"));
        lines.push(format!("#[packed]"));
        lines.push(format!("pub struct {} {{", gen_struct_name(&ty.name)));
        lines.extend(Self::gen_fields(&ty.fields)?);
        lines.push(format!("}}\n"));

        lines.extend(Self::gen_field_impls(&ty.name, &ty.fields)?);

        Ok(lines)
    }

    #[rustfmt::skip]
    fn gen_message(
        msg: &ApiMessage,
        generated_type_set: &mut HashSet<String>,
    ) -> Result<Vec<String>> {
        ensure_not_duplicate!(gen_struct_name(&msg.name), generated_type_set);

        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("#[derive(Pack, Debug, PackDefault)]"));
        lines.push(format!("#[packed]"));
        lines.push(format!("pub struct {} {{", gen_struct_name(&msg.name)));
        lines.extend(Self::gen_fields(&msg.fields)?);
        lines.push(format!("}}\n"));

        // Gen msg crc
        lines.push(format!("impl rsvpp::message::MessageCrc for {} {{", gen_struct_name(&msg.name)));
        lines.push(format!("    fn crc() -> &'static str {{"));
        lines.push(format!("        \"{}\"", msg.extra.crc));
        lines.push(format!("    }}"));
        lines.push(format!("}}\n"));

        lines.extend(Self::gen_field_impls(&msg.name, &msg.fields)?);

        Ok(lines)
    }

    fn gen_union(uni: &ApiUnion, generated_type_set: &mut HashSet<String>) -> Result<Vec<String>> {
        ensure_not_duplicate!(gen_struct_name(&uni.name), generated_type_set);

        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("#[pack_union]"));
        lines.push(format!("#[derive(Debug, PackDefault)]"));
        lines.push(format!("pub union {} {{", gen_struct_name(&uni.name)));
        lines.extend(Self::gen_union_fields(&uni.fields)?);
        lines.push(format!("}}\n"));

        Ok(lines)
    }

    fn gen_union_fields(fields: &Vec<ApiUnionField>) -> Result<Vec<String>> {
        let mut lines: Vec<String> = Vec::new();

        for field in fields {
            lines.push(format!(
                "    {}: {},",
                Self::gen_field_name(&field.name),
                gen_filed_type(&field.ty)
            ));
        }

        Ok(lines)
    }

    fn gen_enum_flag(
        enum_flag: &ApiEnumFlag,
        generated_type_set: &mut HashSet<String>,
    ) -> Result<Vec<String>> {
        ensure_not_duplicate!(gen_struct_name(&enum_flag.name), generated_type_set);

        let mut lines: Vec<String> = Vec::new();

        #[rustfmt::skip]
        lines.push(format!("type {} = {};\n", gen_struct_name(&enum_flag.name), enum_flag.ty));
        for field in &enum_flag.fields {
            #[rustfmt::skip]
            lines.push(format!("const {}: {} = {};\n", field.name, enum_flag.ty, field.value));
        }

        Ok(lines)
    }

    fn gen_enum(enu: &ApiEnum, generated_type_set: &mut HashSet<String>) -> Result<Vec<String>> {
        ensure_not_duplicate!(gen_struct_name(&enu.name), generated_type_set);

        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("#[derive(Pack, Debug, PartialEq, Eq)]"));
        lines.push(format!("#[pack_type(\"{}\")]", enu.ty));
        lines.push(format!("pub enum {} {{", gen_struct_name(&enu.name)));
        lines.extend(Self::gen_enum_fields(&enu.fields)?);

        // Append default field
        lines.push(format!("    #[default]"));
        lines.push(format!("    Mismatch({}),", enu.ty));

        lines.push(format!("}}\n"));

        lines.extend(Self::gen_enum_field_impl(&enu)?);

        Ok(lines)
    }

    fn gen_enum_fields(fields: &Vec<ApiEnumField>) -> Result<Vec<String>> {
        let mut lines: Vec<String> = Vec::new();

        for field in fields {
            lines.push(format!("    #[value({})]", field.value));
            lines.push(format!("    {},", field.name.to_lowercase().hump()));
        }

        Ok(lines)
    }

    fn gen_enum_field_impl(enu: &ApiEnum) -> Result<Vec<String>> {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!(
            "impl PackDefault for {} {{",
            gen_struct_name(&enu.name)
        ));
        lines.push(format!("    fn pack_default() -> Self {{"));
        #[rustfmt::skip]
        lines.push(format!("        Self::{}", enu.fields.get(0).ok_or("Enum is empty")?.name.to_lowercase().hump()));
        lines.push(format!("    }}"));
        lines.push(format!("}}\n"));

        Ok(lines)
    }

    fn gen_fields(fields: &Vec<ApiField>) -> Result<Vec<String>> {
        let mut lines: Vec<String> = Vec::new();

        for field in fields {
            if let Some(refer) = &field.refer {
                lines.push(format!("    #[len(\"{}\")]", refer));
            } else if let Some(n) = field.len {
                if n > 0 {
                    lines.push(format!("    #[len({})]", n));
                }
            }

            lines.push(format!(
                "    {}: {},",
                Self::gen_field_name(&field.name),
                Self::gen_field_type(&field)?
            ));
        }

        Ok(lines)
    }

    fn gen_field_impls(name: &String, fields: &Vec<ApiField>) -> Result<Vec<String>> {
        let struct_name = &gen_struct_name(name);
        let mut lines: Vec<String> = Vec::new();

        // Gen new
        lines.push(format!("impl {} {{", struct_name));
        lines.push(format!("    pub fn new() -> Self {{"));
        lines.push(format!("        Self::pack_default()"));
        lines.push(format!("    }}"));
        lines.push(format!("}}\n"));

        // Gen msg name
        #[rustfmt::skip]
        lines.push(format!("impl rsvpp::message::MessageName for {} {{", struct_name));
        lines.push(format!("    fn message_name() -> String {{"));
        lines.push(format!("        \"{}\".to_string()", name));
        lines.push(format!("    }}"));
        lines.push(format!("}}\n"));

        // Gen fields
        for field in fields {
            let list = match field.name.as_str() {
                #[rustfmt::skip]
                "_vl_msg_id" => Self::gen_field_impl_internal(struct_name, "MessageId", "message_id", field)?,
                #[rustfmt::skip]
                "client_index" => Self::gen_field_impl_internal(struct_name, "MessageClientId", "client_index", field)?,
                #[rustfmt::skip]
                "context" => Self::gen_field_impl_internal(struct_name, "MessageContext", "context", field)?,
                #[rustfmt::skip]
                _ => Self::gen_field_impls_client_other(struct_name, field)?,
            };
            lines.extend(list);
        }

        Ok(lines)
    }

    fn gen_field_impl_internal(
        struct_name: &str,
        impl_name: &str,
        func_name: &str,
        field: &ApiField,
    ) -> Result<Vec<String>> {
        let mut lines: Vec<String> = Vec::new();
        let field_ty = Self::gen_field_type(field)?;
        let field_name = Self::gen_field_name(&field.name);
        let set_func_name = format!("set_{}", func_name).replace("__", "_");

        if BASE_TYPE_SET.contains(field_ty.as_str()) {
            #[rustfmt::skip]
            lines.push(format!( "impl rsvpp::message::{} for {} {{", impl_name, struct_name));
            lines.push(format!("    fn {}(&self) -> {} {{", func_name, field_ty));
            lines.push(format!("        self.{}", field_name));
            lines.push(format!("    }}"));
            #[rustfmt::skip]
            lines.push(format!( "    fn {}(mut self, {}: {}) -> Self {{", set_func_name, field_name, field_ty));
            lines.push(format!("        self.{} = {};", field_name, field_name));
            lines.push(format!("        self"));
            lines.push(format!("    }}"));
            lines.push(format!("}}\n"));
        } else {
            #[rustfmt::skip]
            lines.push(format!( "impl rsvpp::message::{} for {} {{", impl_name, struct_name));
            lines.push(format!("    fn {}(&self) -> &{} {{", func_name, field_ty));
            lines.push(format!("        &self.{}", field_name));
            lines.push(format!("    }}"));
            #[rustfmt::skip]
            lines.push(format!( "    fn {}(mut self, {}: {}) -> Self {{", set_func_name, field_name, field_ty));
            lines.push(format!("        self.{} = {};", field_name, field_name));
            lines.push(format!("        self"));
            lines.push(format!("    }}"));
            lines.push(format!("}}\n"));
        }

        Ok(lines)
    }

    fn gen_field_impls_client_other(struct_name: &String, field: &ApiField) -> Result<Vec<String>> {
        let mut lines: Vec<String> = Vec::new();
        let field_ty = Self::gen_field_type(field)?;
        let field_name = Self::gen_field_name(&field.name);
        let func_name = &field_name;
        let set_func_name = format!("set_{}", func_name).replace("__", "_");

        if BASE_TYPE_SET.contains(field_ty.as_str()) {
            #[rustfmt::skip]
            lines.push(format!("impl {} {{", struct_name));
            #[rustfmt::skip]
            lines.push(format!("    pub fn {}(&self) -> {} {{", func_name, field_ty));
            lines.push(format!("        self.{}", field_name));
            lines.push(format!("    }}"));
            #[rustfmt::skip]
            lines.push(format!("    pub fn {}(mut self, {}: {}) -> Self {{", set_func_name, field_name, field_ty));
            lines.push(format!("        self.{} = {};", field_name, field_name));
            lines.push(format!("        self"));
            lines.push(format!("    }}"));
            lines.push(format!("}}\n"));
        } else {
            #[rustfmt::skip]
            lines.push(format!( "impl {} {{", struct_name));
            lines.push(format!(
                "    pub fn {}(&self) -> &{} {{",
                func_name, field_ty
            ));
            lines.push(format!("        &self.{}", field_name));
            lines.push(format!("    }}"));
            #[rustfmt::skip]
            lines.push(format!("    pub fn {}(mut self, {}: {}) -> Self {{", set_func_name, field_name, field_ty));
            lines.push(format!("        self.{} = {};", field_name, field_name));
            lines.push(format!("        self"));
            lines.push(format!("    }}"));
            lines.push(format!("}}\n"));
        }

        Ok(lines)
    }

    fn gen_field_name(name: &str) -> String {
        if RESERVED_SET.contains(name) {
            format!("_{}", name)
        } else {
            name.to_string()
        }
    }

    fn gen_field_type(field: &ApiField) -> Result<String> {
        // String is special
        if field.ty == "string" {
            return Ok("String".to_string());
        }

        if let Some(n) = field.len {
            if let Some(_) = &field.refer {
                // Dynamic array
                Ok(format!("Vec<{}>", gen_filed_type(&field.ty)))
            } else {
                // Static array
                Ok(format!("[{}; {}]", gen_filed_type(&field.ty), n))
            }
        } else {
            // Not array
            Ok(format!("{}", gen_filed_type(&field.ty)))
        }
    }

    #[rustfmt::skip]
    fn gen_services(name: &String, services: &Vec<ApiService>, has_retval_type_set: &HashSet<String>) -> Result<Vec<String>> {
        // Skip memclnt
        if name == "memclnt" {
            return Ok(Vec::new());
        }

        let struct_name = format!("{}Service", name).hump();
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("pub struct {} {{", struct_name));
        lines.push(format!("    client: std::sync::Arc<rsvpp::Client>,"));
        lines.push(format!("}}\n"));

        lines.push(format!("impl {} {{", struct_name));
        lines.push(format!("    pub fn new(client: std::sync::Arc<rsvpp::Client>) -> Self {{"));
        lines.push(format!("        Self {{ client }}"));
        lines.push(format!("    }}\n"));
        for service in services {
            lines.extend(Self::gen_service(service, has_retval_type_set)?);
        }
        lines.push(format!("}}\n"));

        Ok(lines)
    }

    #[rustfmt::skip]
    fn gen_service(service: &ApiService, has_retval_type_set: &HashSet<String>) -> Result<Vec<String>> {
        let mut lines: Vec<String> = Vec::new();
        let func_name = &service.req;
        let req_type = gen_struct_name(&service.req);
        let rep_type = gen_struct_name(&service.rep);

        if service.is_stream {
            lines.push(format!("    pub async fn {}(&self, req: {}) -> rsvpp::Result<Vec<{}>> {{", func_name, req_type, rep_type));
            lines.push(format!("        let ctx = self.client.send_msg(req).await?;"));
            lines.push(format!("        self.client.send_msg_with_ctx(super::memclnt::ControlPing::new(), ctx).await?;"));

            lines.push(format!("        let mut arr: Vec<{}> = Vec::new();", rep_type));
            lines.push(format!("        'outer: loop {{"));
            lines.push(format!("            for entry in self.client.recv(ctx).await? {{"));
            lines.push(format!("                if entry.header._vl_msg_id == self.client.get_msg_id::<super::memclnt::ControlPingReply>()? {{"));
            lines.push(format!("                    let rep = super::memclnt::ControlPingReply::unpack(&entry.data, 0)?.0;"));
            lines.push(format!("                    check_error(rep.retval() as i32)?;"));
            lines.push(format!("                    break 'outer;"));
            lines.push(format!("                }}"));

            lines.push(format!("                if entry.header._vl_msg_id != self.client.get_msg_id::<{}>()? {{", rep_type));
            lines.push(format!("                    return Err(rsvpp::Error::msg_id_mismatch(\"Message id mismatch in {}\"));", func_name));
            lines.push(format!("                }}"));

            lines.push(format!("                let rep = {}::unpack(&entry.data, 0)?.0;", rep_type));
            if has_retval_type_set.contains(&rep_type) {
                lines.push(format!("                check_error(rep.retval() as i32)?;"));
            }
            lines.push(format!("                arr.push(rep)"));
            lines.push(format!("            }}"));
            lines.push(format!("        }}"));
            lines.push(format!("        Ok(arr)"));
            lines.push(format!("    }}\n"));
        } else {
            lines.push(format!("    pub async fn {}(&self, req: {}) -> rsvpp::Result<{}> {{", func_name, req_type, rep_type));
            lines.push(format!("        let ctx = self.client.send_msg(req).await?;"));
            lines.push(format!("        let rep: {} = self.client.recv_msg(ctx).await?;", rep_type));
            if has_retval_type_set.contains(&rep_type) {
                lines.push(format!("        check_error(rep.retval() as i32)?;"));
            }
            lines.push(format!("        Ok(rep)"));
            lines.push(format!("    }}\n"));
        }

        Ok(lines)
    }
}

fn gen_filed_type(ty: &str) -> String {
    if BASE_TYPE_SET.contains(ty) {
        ty.to_string()
    } else {
        if ty.starts_with("vl_api_") {
            // Cleanup vl_api_XXX_t
            ty[7..ty.len() - 2].to_string().hump()
        } else {
            ty.to_string().hump()
        }
    }
}

fn gen_struct_name(name: &str) -> String {
    name.to_string().hump()
}
