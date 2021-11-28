use std::str::FromStr;

use proc_macro2::TokenStream;

/// Parse token stream to syn type, call proc_macro_error::abort! if error occurred
///
/// ```rust
/// let list: syn::ExprTuple = parse2! { attr.tokens,
///     "Syntax error of module imports";
///     note = "Syntax is #[imports(MODULE_A as TYPE_A, MODULEB as TYPE_B,)]";
/// };
/// ```
macro_rules! parse2 {
    ($tokens:expr, $($tts:tt)*) => {
        match syn::parse2($tokens.clone()) {
            Ok(v) => v,
            Err(_) => {
                proc_macro_error::abort! { $tokens,
                    $($tts)*
                };
            },
        }
    };
}

pub fn str_to_toks<S: AsRef<str>>(s: S) -> TokenStream {
    TokenStream::from_str(s.as_ref())
        .map_err(|e| format!("Parse '{}' error: {}", s.as_ref(), e))
        .unwrap()
}

/// Parse token stream like ("foo")
pub fn parse_string_arg(item: &TokenStream) -> Result<String, String> {
    let s = (move || -> Result<String, ()> {
        let expr = syn::parse2::<syn::ExprParen>(item.clone()).map_err(|_| ())?;
        let lit = match *expr.expr {
            syn::Expr::Lit(lit) => lit,
            _ => return Err(()),
        };
        let s = match lit.lit {
            syn::Lit::Str(s) => s.value(),
            _ => return Err(()),
        };

        Ok(s)
    })()
    .map_err(|_| format!("Parse token error, expect (\"\\w*\")"))?;

    Ok(s)
}
