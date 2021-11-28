use serde_json::Value;

use crate::Result;

#[derive(Debug)]
pub struct ApiField {
    pub ty: String,
    pub name: String,
    pub len: Option<usize>,
    pub refer: Option<String>,
}

#[derive(Debug)]
pub struct ApiType {
    pub name: String,
    pub fields: Vec<ApiField>,
}

#[derive(Debug)]
pub struct ApiMessageExtra {
    pub crc: String,
}

#[derive(Debug)]
pub struct ApiMessage {
    pub name: String,
    pub fields: Vec<ApiField>,
    pub extra: ApiMessageExtra,
}

#[derive(Debug)]
pub struct ApiUnionField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug)]
pub struct ApiUnion {
    pub name: String,
    pub fields: Vec<ApiUnionField>,
}

#[derive(Debug)]
pub struct ApiEnumField {
    pub name: String,
    pub value: usize,
}

#[derive(Debug)]
pub struct ApiEnum {
    pub name: String,
    pub ty: String,
    pub fields: Vec<ApiEnumField>,
}

#[derive(Debug)]
pub struct ApiService {
    pub req: String,
    pub rep: String,
    pub is_stream: bool,
}

#[derive(Debug)]
pub struct ApiAliase {
    pub name: String,
    pub ty: String,
    pub len: Option<usize>,
}

#[derive(Debug)]
pub struct JsonApi {
    pub name: String,
    pub types: Vec<ApiType>,
    pub messages: Vec<ApiMessage>,
    pub unions: Vec<ApiUnion>,
    pub enums: Vec<ApiEnum>,
    pub services: Vec<ApiService>,
    pub aliases: Vec<ApiAliase>,
}

impl JsonApi {
    pub fn parse(name: String, content: &str) -> Result<Self> {
        let value: Value = serde_json::from_str(content)?;
        let mut instance = Self {
            name,
            types: Vec::new(),
            messages: Vec::new(),
            unions: Vec::new(),
            enums: Vec::new(),
            services: Vec::new(),
            aliases: Vec::new(),
        };

        instance.parse_types(&value)?;
        instance.parse_messages(&value)?;
        instance.parse_unions(&value)?;
        instance.parse_enums(&value)?;
        instance.parse_services(&value)?;
        instance.parse_aliases(&value)?;

        Ok(instance)
    }

    fn parse_types(&mut self, value: &Value) -> Result<()> {
        if let Some(Value::Array(types)) = value.get("types") {
            for value in types {
                let arr = value.as_array().ok_or("Type must be array")?;

                // Check length
                if arr.len() < 1 {
                    return Err(format!("Type array length must > 0: '{}'", value).into());
                }

                // Parse type name
                let name = arr[0].as_str().ok_or("Type name must be str")?.to_string();

                // Parse fields
                let mut fields: Vec<ApiField> = Vec::new();
                for v in &arr[1..] {
                    fields.push(Self::parse_field(v)?);
                }

                // Append type
                self.types.push(ApiType { name, fields });
            }
        } else {
            return Err("Types not array".into());
        }

        Ok(())
    }

    fn parse_messages(&mut self, value: &Value) -> Result<()> {
        if let Some(Value::Array(msgs)) = value.get("messages") {
            for value in msgs {
                let arr = value.as_array().ok_or("Message must be array")?;

                // Check length
                if arr.len() < 2 {
                    return Err(format!("Message array length must > 1: '{}'", value).into());
                }

                // Parse message name
                let name = arr[0]
                    .as_str()
                    .ok_or("Message name must be str")?
                    .to_string();

                // Parse message extra
                let extra = Self::parse_message_extra(&arr[arr.len() - 1])?;

                // Parse fields
                let mut fields: Vec<ApiField> = Vec::new();
                for v in &arr[1..arr.len() - 1] {
                    fields.push(Self::parse_field(v)?);
                }

                // Append message
                self.messages.push(ApiMessage {
                    name,
                    fields,
                    extra,
                });
            }
        } else {
            return Err("Messages not array".into());
        }

        Ok(())
    }

    fn parse_unions(&mut self, value: &Value) -> Result<()> {
        if let Some(Value::Array(unions)) = value.get("unions") {
            for uni in unions {
                let mut name = "";
                let mut fields: Vec<ApiUnionField> = Vec::new();

                for item in uni.as_array().ok_or("Union must be array")? {
                    // Parse name
                    if let Some(n) = item.as_str() {
                        name = n;
                    }

                    // Parse field
                    if let Some(arr) = item.as_array() {
                        if let Some(Value::String(ty)) = arr.get(0) {
                            if let Some(Value::String(name)) = arr.get(1) {
                                fields.push(ApiUnionField {
                                    name: name.to_string(),
                                    ty: ty.to_string(),
                                });
                            } else {
                                return Err("Enum field array index 1 must be string".into());
                            }
                        } else {
                            return Err("Enum field array index 0 must be string".into());
                        }
                    }
                }

                self.unions.push(ApiUnion {
                    name: name.to_string(),
                    fields,
                });
            }
        } else {
            return Err("Unions not array".into());
        }

        Ok(())
    }

    fn parse_enums(&mut self, value: &Value) -> Result<()> {
        if let Some(Value::Array(enums)) = value.get("enums") {
            for enu in enums {
                let mut name = "";
                let mut ty = "";
                let mut fields: Vec<ApiEnumField> = Vec::new();

                for item in enu.as_array().ok_or("Enum must be array")? {
                    // Parse name
                    if let Some(n) = item.as_str() {
                        name = n;
                    }

                    // Parse type
                    if let Some(obj) = item.as_object() {
                        if let Some(Value::String(t)) = obj.get("enumtype") {
                            ty = t;
                        } else {
                            return Err("Enum object missing type".into());
                        }
                    }

                    // Parse field
                    if let Some(arr) = item.as_array() {
                        if let Some(Value::String(name)) = arr.get(0) {
                            if let Some(Value::Number(n)) = arr.get(1) {
                                if let Some(n) = n.as_u64() {
                                    fields.push(ApiEnumField {
                                        name: name.to_string(),
                                        value: n as usize,
                                    });
                                } else {
                                    return Err("Enum field array index 1 must be u64".into());
                                }
                            } else {
                                return Err("Enum field array index 1 must be number".into());
                            }
                        } else {
                            return Err("Enum field array index 0 must be string".into());
                        }
                    }
                }

                self.enums.push(ApiEnum {
                    name: name.to_string(),
                    ty: ty.to_string(),
                    fields,
                });
            }
        } else {
            return Err("Enums not array".into());
        }

        Ok(())
    }

    fn parse_services(&mut self, value: &Value) -> Result<()> {
        if let Some(Value::Object(services)) = value.get("services") {
            for (k, v) in services {
                let req = k.clone();
                let rep = v
                    .get("reply")
                    .ok_or(format!("Replay not found: '{}'", v))?
                    .as_str()
                    .ok_or("Replay must be string")?
                    .to_string();
                let is_stream = if let Some(s) = v.get("stream") {
                    s.as_bool().ok_or("stream must be bool")?
                } else {
                    false
                };

                self.services.push(ApiService {
                    req,
                    rep,
                    is_stream,
                });
            }
        } else {
            return Err("Services not object".into());
        }

        Ok(())
    }

    fn parse_aliases(&mut self, value: &Value) -> Result<()> {
        if let Some(Value::Object(aliases)) = value.get("aliases") {
            for (k, v) in aliases {
                let name = k.clone();
                let ty = v
                    .get("type")
                    .ok_or(format!("Type not found: '{}'", v))?
                    .as_str()
                    .ok_or("Type must be string")?
                    .to_string();
                let len = if let Some(s) = v.get("length") {
                    Some(s.as_i64().ok_or("length must be int")? as usize)
                } else {
                    None
                };

                self.aliases.push(ApiAliase { name, ty, len });
            }
        } else {
            return Err("Aliases not object".into());
        }

        Ok(())
    }

    fn parse_field(value: &Value) -> Result<ApiField> {
        let arr = value.as_array().ok_or(format!("Expect array: {}", value))?;
        let ty = arr
            .get(0)
            .ok_or("Missing type")?
            .as_str()
            .ok_or("Expect str")?
            .to_string();
        let name = arr
            .get(1)
            .ok_or("Missing type")?
            .as_str()
            .ok_or("Expect str")?
            .to_string();
        let len = if let Some(v) = arr.get(2) {
            v.as_i64().map(|v| v as usize)
        } else {
            None
        };
        let refer = if let Some(v) = arr.get(3) {
            Some(v.as_str().ok_or("Expect str")?.to_string())
        } else {
            None
        };

        Ok(ApiField {
            ty,
            name,
            len,
            refer,
        })
    }

    fn parse_message_extra(value: &Value) -> Result<ApiMessageExtra> {
        let crc = value
            .get("crc")
            .ok_or(format!("Crc not found: '{}'", value))?
            .as_str()
            .ok_or("Crc must be str")?
            .to_string();

        Ok(ApiMessageExtra { crc })
    }
}
