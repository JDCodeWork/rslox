use crate::{
    dbg::dbg_token,
    scanner::{Scanner, TokenKind},
};

pub fn compile(source: &str) {
    let mut sc = Scanner::new(source);

    let mut ln = 0;
    loop {
        let tok = match sc.scan_token() {
            Err(err) => break eprintln!("{err}"),
            Ok(t) => t,
        };

        dbg_token(&tok, &mut ln, source); // Debug mode (--features dbg)

        if tok.kind == TokenKind::EOF {
            break;
        }
    }
}
