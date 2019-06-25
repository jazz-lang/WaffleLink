use std::mem;

use super::{ast::Location, ast::*, lexer::*};
use crate::reader::Reader;
use crate::{
    err::{Msg, MsgWithPos},
    *,
};
use std::cell::RefCell;
use std::collections::HashSet;

pub struct Parser<'a> {
    lexer: Lexer,
    token: Token,
    pub ast: &'a mut File,
}

pub struct IdGen {
    counter: std::cell::RefCell<usize>,
}

lazy_static::lazy_static! {
    pub static ref ID_GEN: std::sync::Mutex<IdGen> = std::sync::Mutex::new(IdGen {
        counter: std::cell::RefCell::new(0)
    });
}

pub fn gen_id() -> usize {
    let mut write = ID_GEN.lock().unwrap();
    let counter = *write.counter.borrow();
    *write.counter.borrow_mut() += 1;

    counter
}

pub type NodeId = usize;
type ExprResult = Result<Box<Expr>, MsgWithPos>;
type StmtResult = Result<Box<StmtKind>, MsgWithPos>;

impl<'a> Parser<'a> {
    pub fn new(reader: Reader, ast: &'a mut File) -> Parser<'a> {
        let token = Token::new(TokenKind::End, Location::new(&reader.filename, 1, 1));
        let lexer = Lexer::new(reader);
        Parser { lexer, token, ast }
    }

    fn generate_id(&self) -> NodeId {
        gen_id()
    }
    pub fn src(&self) -> String {
        self.lexer.reader.src.clone()
    }

    pub fn parse_statement(&mut self) -> StmtResult {
        match &self.token.kind.clone() {
            TokenKind::Var => self.parse_var(),
            TokenKind::LBrace => self.parse_block(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Break => self.parse_break(),
            TokenKind::Continue => self.parse_continue(),
            TokenKind::Else => Err(MsgWithPos::new(
                self.lexer.path().to_string(),
                self.src(),
                self.token.position.clone(),
                Msg::MisplacedElse,
            )),

            _ => self.parse_expression_statement(),
        }
    }

    fn parse_global(
        &mut self,
        _modifiers: &HashSet<String>,
        elements: &mut Vec<Element>,
    ) -> Result<(), MsgWithPos> {
        let pos = self.token.position.clone();

        self.advance_token()?;
        let name = self.expect_identifier()?;

        self.expect_token(TokenKind::Colon)?;
        let data_type = self.parse_type()?;
        if self.token.is(TokenKind::Semicolon) {
            self.expect_semicolon()?;
        }
        elements.push(Element::Var(ExternVar {
            pos,
            name,
            ty: box data_type,
        }));

        Ok(())
    }

    fn parse_modifiers(&mut self) -> Result<HashSet<String>, MsgWithPos> {
        let mut modifiers = HashSet::new();
        loop {
            let modifier = match self.token.kind {
                TokenKind::Inline => "inline",
                TokenKind::Extern => "extern",
                TokenKind::Internal => "internal",
                TokenKind::Pub => "pub",
                TokenKind::Static => "static",
                //TokenKind::ConstExpr => "constant",
                _ => {
                    break;
                }
            };

            if modifiers.contains(modifier) {
                return Err(MsgWithPos::new(
                    self.lexer.path().to_string(),
                    self.src(),
                    self.token.position.clone(),
                    Msg::RedundantModifier(self.token.name()),
                ));
            }

            let _ = self.advance_token()?.position;
            modifiers.insert(modifier.to_owned());
        }

        Ok(modifiers)
    }
    fn init(&mut self) -> Result<(), MsgWithPos> {
        self.advance_token()?;

        Ok(())
    }
    pub fn parse(&mut self) -> Result<(), MsgWithPos> {
        self.init()?;
        let mut elements = vec![];

        while !self.token.is_eof() {
            self.parse_top_level_element(&mut elements)?;
        }
        if !self.src().is_empty() {
            self.ast.src = self.src();
        }
        self.ast.ast.append(&mut elements);

        Ok(())
    }

    pub fn parse_top_level_element(
        &mut self,
        elements: &mut Vec<Element>,
    ) -> Result<(), MsgWithPos> {
        let modifiers = self.parse_modifiers()?;

        match &self.token.kind {
            TokenKind::Import => {
                self.advance_token()?;
                elements.push(self.parse_import()?);
            }
            TokenKind::Var => {
                self.parse_global(&modifiers, elements)?;
            }
            TokenKind::Fun => {
                let fun = self.parse_function(modifiers)?;
                elements.push(Element::Func(fun));
            }

            TokenKind::Const => {
                let constant = self.parse_const()?;
                elements.push(Element::Const(constant));
            }

            TokenKind::Struct => {
                let struc = self.parse_struct(false)?;

                elements.push(Element::Struct(struc))
            }

            _ => {
                let msg = Msg::ExpectedTopLevelElement(self.token.name());
                return Err(MsgWithPos::new(
                    self.lexer.path().to_string(),
                    self.src(),
                    self.token.position.clone(),
                    msg,
                ));
            }
        }
        Ok(())
    }
    #[allow(dead_code)]
    fn parse_const(&mut self) -> Result<Constant, MsgWithPos> {
        let pos = self.expect_token(TokenKind::Const)?.position;
        let name = self.expect_identifier()?;
        let ty = if self.token.is(TokenKind::Colon) {
            self.advance_token()?;
            Some(box self.parse_type()?)
        } else {
            None
        };
        self.expect_token(TokenKind::Eq)?;
        let expr = self.parse_expression()?;
        if self.token.is(TokenKind::Semicolon) {
            self.expect_semicolon()?;
        }

        Ok(Constant {
            id: self.generate_id(),
            pos,
            name,
            ty: ty,
            value: expr,
        })
    }

    fn parse_struct(&mut self, _: bool) -> Result<Struct, MsgWithPos> {
        let pos = self.expect_token(TokenKind::Struct)?.position;

        let ident = self.expect_identifier()?;

        self.expect_token(TokenKind::LBrace)?;
        let fields = self.parse_comma_list(TokenKind::RBrace, |p| p.parse_struct_field())?;

        Ok(Struct {
            id: self.generate_id(),
            name: ident,
            loc: pos,
            fields: fields,
        })
    }

    fn parse_struct_field(&mut self) -> Result<(Location, String, Box<Type>), MsgWithPos> {
        let pos = self.token.position.clone();
        let ident = self.expect_identifier()?;

        let ty = self.parse_type()?;

        Ok((pos, ident, box ty))
    }

    fn parse_function_block(&mut self) -> Result<Option<Box<StmtKind>>, MsgWithPos> {
        if self.token.is(TokenKind::Semicolon) {
            self.advance_token()?;

            Ok(None)
        } else if self.token.is(TokenKind::Arrow) {
            let expr = self.parse_function_block_expression()?;

            Ok(Some(expr))
        } else {
            let block = self.parse_block()?;

            Ok(Some(block))
        }
    }

    fn parse_function_block_expression(&mut self) -> Result<Box<StmtKind>, MsgWithPos> {
        self.advance_token()?;
        let _ = self.token.position.clone();

        match self.token.kind {
            TokenKind::Return => self.parse_return(),
            _ => {
                let expr = self.parse_expression().ok();
                self.expect_token(TokenKind::Semicolon)?;
                Ok(Box::new(StmtKind::Return(expr)))
            }
        }
    }

    fn parse_return(&mut self) -> StmtResult {
        let _ = self.expect_token(TokenKind::Return)?.position;
        let expr = if self.token.is(TokenKind::Semicolon) {
            None
        } else {
            let expr = self.parse_expression()?;
            Some(expr)
        };

        if self.token.is(TokenKind::Semicolon) {
            self.expect_semicolon()?;
        }

        Ok(Box::new(StmtKind::Return(expr)))
    }

    fn parse_block(&mut self) -> StmtResult {
        let _ = self.expect_token(TokenKind::LBrace)?.position;
        let mut stmts = vec![];

        while !self.token.is(TokenKind::RBrace) && !self.token.is_eof() {
            let stmt = self.parse_statement()?;
            stmts.push(stmt);
        }

        self.expect_token(TokenKind::RBrace)?;

        Ok(Box::new(StmtKind::Block(stmts)))
    }

    fn parse_var(&mut self) -> StmtResult {
        let _ = self.advance_token()?.position;
        let ident = self.expect_identifier()?;
        let data_type = self.parse_var_type()?;
        let expr = self.parse_var_assignment()?;
        if self.token.is(TokenKind::Semicolon) {
            self.expect_semicolon()?;
        }

        Ok(Box::new(StmtKind::VarDecl(ident, data_type, expr)))
    }
    fn parse_var_assignment(&mut self) -> Result<Option<Box<Expr>>, MsgWithPos> {
        if self.token.is(TokenKind::Eq) {
            self.expect_token(TokenKind::Eq)?;
            let expr = self.parse_expression()?;

            Ok(Some(expr))
        } else {
            Ok(None)
        }
    }

    fn parse_var_type(&mut self) -> Result<Option<Box<Type>>, MsgWithPos> {
        if !self.token.is(TokenKind::Eq) {
            Ok(Some(box self.parse_type()?))
        } else {
            Ok(None)
        }
    }

    fn advance_token(&mut self) -> Result<Token, MsgWithPos> {
        let tok = self.lexer.read_token()?;

        Ok(mem::replace(&mut self.token, tok))
    }
    fn expect_semicolon(&mut self) -> Result<Token, MsgWithPos> {
        self.expect_token(TokenKind::Semicolon)
    }

    fn expect_token(&mut self, kind: TokenKind) -> Result<Token, MsgWithPos> {
        if self.token.kind == kind {
            let token = self.advance_token()?;

            Ok(token)
        } else {
            Err(MsgWithPos::new(
                self.lexer.path().to_string(),
                self.src(),
                self.token.position.clone(),
                Msg::ExpectedToken(kind.name().into(), self.token.name()),
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, MsgWithPos> {
        let tok = self.advance_token()?;

        if let TokenKind::Identifier(ref value) = tok.kind {
            return Ok(value.to_owned());
        } else {
            Err(MsgWithPos::new(
                self.lexer.path().to_string(),
                self.src(),
                tok.position.clone(),
                Msg::ExpectedIdentifier(tok.name()),
            ))
        }
    }

    fn parse_null(&mut self) -> ExprResult {
        let tok = self.advance_token()?;
        Ok(Box::new(Expr {
            id: self.generate_id(),
            pos: tok.position,
            kind: ExprKind::Null,
        }))
    }
    fn parse_bool_literal(&mut self) -> ExprResult {
        let tok = self.advance_token()?;
        let value = tok.is(TokenKind::True);

        Ok(Box::new(Expr {
            id: self.generate_id(),
            pos: tok.position,
            kind: ExprKind::Bool(value),
        }))
    }

    fn parse_string(&mut self) -> ExprResult {
        let string = self.advance_token()?;

        if let TokenKind::String(value) = string.kind {
            Ok(Box::new(Expr {
                id: self.generate_id(),
                pos: string.position,
                kind: ExprKind::String(value),
            }))
        } else {
            unreachable!();
        }
    }

    fn parse_lit_float(&mut self) -> ExprResult {
        let tok = self.advance_token()?;
        let pos = tok.position;

        if let TokenKind::LitFloat(value, suffix) = tok.kind {
            let filtered = value.chars().filter(|&ch| ch != '_').collect::<String>();
            let parsed = filtered.parse::<f64>();

            if let Ok(num) = parsed {
                let expr = Expr {
                    id: self.generate_id(),
                    pos,
                    kind: ExprKind::Float(num, suffix),
                };
                return Ok(Box::new(expr));
            }
        }

        unreachable!()
    }

    fn parse_lit_char(&mut self) -> ExprResult {
        let tok = self.advance_token()?;
        let pos = tok.position;

        if let TokenKind::LitChar(val) = tok.kind {
            Ok(Box::new(Expr {
                id: self.generate_id(),
                pos,
                kind: ExprKind::Character(val),
            }))
        } else {
            unreachable!();
        }
    }

    fn parse_lit_int(&mut self) -> ExprResult {
        let tok = self.advance_token()?;
        let pos = tok.position;

        if let TokenKind::LitInt(value, base, suffix) = tok.kind {
            let filtered = value.chars().filter(|&ch| ch != '_').collect::<String>();
            let parsed = u64::from_str_radix(&filtered, base.num());

            match parsed {
                Ok(num) => {
                    let expr = Expr {
                        id: self.generate_id(),
                        pos,
                        kind: ExprKind::Integer(num, suffix),
                    };
                    Ok(Box::new(expr))
                }

                _ => {
                    let bits = match suffix {
                        IntSuffix::Byte => "byte",
                        IntSuffix::Int => "int",
                        IntSuffix::Long => "long",
                        IntSuffix::ULong => "ulong",
                        IntSuffix::UInt => "uint",
                        IntSuffix::UByte => "ubyte",
                    };

                    Err(MsgWithPos::new(
                        self.lexer.path().to_string(),
                        self.src(),
                        pos,
                        Msg::NumberOverflow(bits.into()),
                    ))
                }
            }
        } else {
            unreachable!();
        }
    }

    fn parse_expression_statement(&mut self) -> StmtResult {
        let _ = self.token.position.clone();
        let expr = self.parse_expression()?;
        if self.token.is(TokenKind::Semicolon) {
            self.expect_semicolon()?;
        }

        Ok(Box::new(StmtKind::Expr(expr)))
    }

    fn parse_expression(&mut self) -> ExprResult {
        let opts = ExprParsingOpts::new();
        self.parse_binary(0, &opts)
    }

    fn parse_expression_with_opts(&mut self, opts: &ExprParsingOpts) -> ExprResult {
        self.parse_binary(0, opts)
    }

    fn parse_call(&mut self, pos: Location, object: Option<Box<Expr>>, path: String) -> ExprResult {
        self.expect_token(TokenKind::LParen)?;

        let args = self.parse_comma_list(TokenKind::RParen, |p| p.parse_expression())?;

        Ok(Box::new(Expr {
            id: self.generate_id(),
            pos,
            kind: ExprKind::Call(path, object, args),
        }))
    }

    fn parse_identifier_or_call(&mut self, opts: &ExprParsingOpts) -> ExprResult {
        let pos = self.token.position.clone();
        let mut path = vec![self.expect_identifier()?];

        while self.token.is(TokenKind::Sep) {
            self.advance_token()?;
            let ident = self.expect_identifier()?;
            path.push(ident);
        }

        // is this a function call?
        if self.token.is(TokenKind::LParen) {
            self.parse_call(pos, None, path.first().unwrap().clone())
        } else if self.token.is(TokenKind::LBrace) && opts.parse_struct_lit {
            self.parse_lit_struct(pos, path.first().unwrap().clone())

        // if not we have a simple identifier
        } else {
            assert_eq!(1, path.len());
            let name = path[0].clone();
            Ok(Box::new(Expr {
                id: self.generate_id(),
                pos,
                kind: ExprKind::Identifier(name),
            }))
        }
    }

    fn parse_lit_struct(&mut self, pos: Location, path: String) -> ExprResult {
        self.expect_token(TokenKind::LBrace)?;
        let args = self.parse_comma_list(TokenKind::RBrace, |p| p.parse_lit_struct_arg())?;

        Ok(Box::new(Expr {
            id: self.generate_id(),
            pos,
            kind: ExprKind::StructConstruct(path, args),
        }))
    }

    fn parse_lit_struct_arg(&mut self) -> Result<(String, Box<Expr>), MsgWithPos> {
        let _ = self.token.position.clone();
        let name = self.expect_identifier()?;

        self.expect_token(TokenKind::Colon)?;

        let expr = self.parse_expression()?;

        Ok((name, expr))
    }

    fn parse_binary(&mut self, precedence: u32, opts: &ExprParsingOpts) -> ExprResult {
        let mut left = self.parse_unary(opts)?;

        loop {
            let right_precedence = match self.token.kind {
                TokenKind::Or => 1,
                TokenKind::And => 2,
                TokenKind::Eq => 3,
                TokenKind::EqEq
                | TokenKind::Ne
                | TokenKind::Lt
                | TokenKind::Le
                | TokenKind::Gt
                | TokenKind::Ge => 4,
                TokenKind::EqEqEq | TokenKind::NeEqEq => 5,
                TokenKind::BitOr | TokenKind::BitAnd | TokenKind::Caret => 6,
                TokenKind::LtLt | TokenKind::GtGt | TokenKind::GtGtGt => 7,
                TokenKind::Add | TokenKind::Sub => 8,
                TokenKind::Mul | TokenKind::Div | TokenKind::Mod => 9,
                TokenKind::Is | TokenKind::As => 10,
                _ => {
                    return Ok(left);
                }
            };

            if precedence >= right_precedence {
                return Ok(left);
            }

            let tok = self.advance_token()?;

            left = match tok.kind {
                TokenKind::As => {
                    let right = Box::new(self.parse_type()?);
                    let expr = Expr {
                        id: self.generate_id(),
                        pos: tok.position,
                        kind: ExprKind::Conv(left, right),
                    };

                    Box::new(expr)
                }

                _ => {
                    let right = self.parse_binary(right_precedence, opts)?;
                    self.create_binary(tok, left, right)
                }
            };
        }
    }

    fn parse_sizeof(&mut self) -> ExprResult {
        let tok = self.expect_token(TokenKind::SizeOf)?;
        let expect_rparen = if self.token.is(TokenKind::LParen) {
            self.advance_token()?;
            true
        } else {
            false
        };
        let ty = self.parse_type()?;

        if expect_rparen {
            self.expect_token(TokenKind::RParen)?;
        }

        Ok(Box::new(Expr {
            pos: tok.position,
            id: self.generate_id(),
            kind: ExprKind::SizeOf(Box::new(ty)),
        }))
    }

    fn parse_primary(&mut self, opts: &ExprParsingOpts) -> ExprResult {
        let mut left = self.parse_factor(opts)?;
        loop {
            left = match self.token.kind {
                TokenKind::Dot => {
                    let tok = self.advance_token()?;
                    let ident = self.expect_identifier()?;
                    if self.token.is(TokenKind::LParen) {
                        self.parse_call(tok.position, Some(left), ident)?
                    } else {
                        Box::new(Expr {
                            pos: tok.position,
                            id: self.generate_id(),
                            kind: ExprKind::Member(left, ident),
                        })
                    }
                }
                TokenKind::LBracket => {
                    let tok = self.advance_token()?;
                    let index = self.parse_expression()?;
                    self.expect_token(TokenKind::RBracket)?;

                    Box::new(Expr {
                        pos: tok.position,
                        id: self.generate_id(),
                        kind: ExprKind::Subscript(left, index),
                    })
                }

                _ => return Ok(left),
            }
        }
    }

    fn parse_import(&mut self) -> Result<Element, MsgWithPos> {
        if let TokenKind::String(s) = &self.token.kind.clone() {
            let pos = self.advance_token()?.position;
            return Ok(Element::Import(Import {
                pos: pos,
                name: s.to_owned(),
            }));
        } else {
            unimplemented!()
            //Err(MsgWithPos::new(self.lexer.reader.path().to_owned(),self.src(),
            // self.token.pos,Msg::))
        }
    }

    fn parse_function(&mut self, modifiers: HashSet<String>) -> Result<Function, MsgWithPos> {
        let pos = self.expect_token(TokenKind::Fun)?.position;
        let mut variadic = false;

        let this_ = if self.token.is(TokenKind::LParen) {
            self.advance_token()?;
            let name = self.expect_identifier()?;
            let this_ty = self.parse_type()?;
            self.expect_token(TokenKind::RParen)?;
            Some((name, Box::new(this_ty)))
        } else {
            None
        };
        let ident = self.expect_identifier()?;
        self.expect_token(TokenKind::LParen)?;

        /*let params = self.parse_comma_list(TokenKind::RParen, |p| {
            let name = self.expect_identifier()?;
            self.expect_token(TokenKind::Colon);
            let ty = self.parse_type()?;

            Ok((name,Box::new(ty)))
        });*/
        let params = {
            let mut data = vec![];
            let mut comma = true;
            let stop = TokenKind::RParen;
            while !self.token.is(stop.clone()) && !self.token.is_eof() {
                if !comma {
                    return Err(MsgWithPos::new(
                        self.lexer.path().to_string(),
                        self.src(),
                        self.token.position.clone(),
                        Msg::ExpectedToken(TokenKind::Comma.name().into(), self.token.name()),
                    ));
                }

                //self.advance_token()?;

                if self.token.is(TokenKind::DotDotDot) {
                    assert!(!variadic);
                    self.advance_token()?;
                    variadic = true;

                    break;
                }

                let entry = {
                    let name = self.expect_identifier()?;
                    let ty = self.parse_type()?;
                    Ok((name, Box::new(ty)))
                };
                data.push(entry?);

                comma = self.token.is(TokenKind::Comma);
                if comma {
                    self.advance_token()?;
                }
            }

            self.expect_token(stop)?;

            data
        };

        let ty = self.parse_type()?;
        let body = if modifiers.contains("extern") || modifiers.contains("internal") {
            if self.token.is(TokenKind::Semicolon) {
                self.expect_semicolon()?;
            }
            None
        } else {
            self.parse_function_block()?
        };

        Ok(Function {
            id: self.generate_id(),
            name: ident,
            pos,
            internal: modifiers.contains("internal"),
            external: modifiers.contains("extern"),
            this: this_,
            returns: Box::new(ty),
            parameters: params,
            variadic,
            body,
        })
    }

    fn parse_if(&mut self) -> StmtResult {
        let _ = self.expect_token(TokenKind::If)?.position;

        let mut opts = ExprParsingOpts::new();
        opts.parse_struct_lit(false);
        let cond = self.parse_expression_with_opts(&opts)?;

        let then_block = self.parse_block()?;

        let else_block = if self.token.is(TokenKind::Else) {
            self.advance_token()?;

            if self.token.is(TokenKind::If) {
                let if_block = self.parse_if()?;
                //let block = Stmt::create_block(self.generate_id(), if_block.pos(),
                // vec![if_block]);
                let block = StmtKind::Block(vec![if_block]);

                Some(Box::new(block))
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(Box::new(StmtKind::If(cond, then_block, else_block)))
        //Ok(Box::new(Stmt::create_if(self.generate_id(), pos, cond, then_block,
        // else_block)))
    }

    fn parse_for(&mut self) -> StmtResult {
        let _ = self.expect_token(TokenKind::For)?.position;

        let mut opts = ExprParsingOpts::new();
        opts.parse_struct_lit(false);
        let var = self.parse_var()?;
        self.expect_token(TokenKind::Comma)?;
        let cond = self.parse_expression()?;
        self.expect_token(TokenKind::Comma)?;
        let then = self.parse_expression()?;

        let body = self.parse_statement()?;

        Ok(Box::new(StmtKind::For(var, cond, then, body)))
    }

    fn parse_while(&mut self) -> StmtResult {
        let _ = self.expect_token(TokenKind::While)?.position;

        let mut opts = ExprParsingOpts::new();
        opts.parse_struct_lit(false);
        let expr = self.parse_expression_with_opts(&opts)?;

        let block = self.parse_block()?;

        Ok(Box::new(StmtKind::While(expr, block)))
    }

    fn parse_break(&mut self) -> StmtResult {
        let _ = self.expect_token(TokenKind::Break)?.position;
        if self.token.is(TokenKind::Semicolon) {
            self.expect_semicolon()?;
        }

        Ok(Box::new(StmtKind::Break))
    }

    fn parse_continue(&mut self) -> StmtResult {
        let _ = self.expect_token(TokenKind::Continue)?.position;
        if self.token.is(TokenKind::Semicolon) {
            self.expect_semicolon()?;
        }

        Ok(Box::new(StmtKind::Continue))
    }

    fn parse_comma_list<F, R>(
        &mut self,
        stop: TokenKind,
        mut parse: F,
    ) -> Result<Vec<R>, MsgWithPos>
    where
        F: FnMut(&mut Parser<'_>) -> Result<R, MsgWithPos>,
    {
        let mut data = vec![];
        let mut comma = true;

        while !self.token.is(stop.clone()) && !self.token.is_eof() {
            if !comma {
                return Err(MsgWithPos::new(
                    self.lexer.path().to_string(),
                    self.src(),
                    self.token.position.clone(),
                    Msg::ExpectedToken(TokenKind::Comma.name().into(), self.token.name()),
                ));
            }

            let entry = parse(self)?;
            data.push(entry);

            comma = self.token.is(TokenKind::Comma);
            if comma {
                self.advance_token()?;
            }
        }

        self.expect_token(stop)?;

        Ok(data)
    }

    fn parse_type(&mut self) -> Result<Type, MsgWithPos> {
        let ty = match self.token.kind {
            TokenKind::Identifier(_) => {
                let pos = self.token.position.clone();
                let name = self.expect_identifier()?;
                if &name == "void" {
                    return Ok(Type::new(pos, TypeKind::Void));
                }

                Type::new(pos, TypeKind::Basic(name))
            }

            TokenKind::Mul => {
                let pos = self.token.position.clone();
                self.advance_token()?;
                let subty = self.parse_type()?;
                Type::new(pos, TypeKind::Pointer(box subty))
            }

            TokenKind::LParen => {
                let token = self.advance_token()?;
                let subtypes = self.parse_comma_list(TokenKind::RParen, |p| {
                    let ty = p.parse_type()?;

                    Ok(Box::new(ty))
                })?;

                self.expect_token(TokenKind::Arrow)?;
                //self.advance_token()?;
                let ret = Box::new(self.parse_type()?);

                Type::new(token.position.clone(), TypeKind::Function(ret, subtypes))
            }

            _ => {
                return Err(MsgWithPos::new(
                    self.lexer.path().to_string(),
                    self.src(),
                    self.token.position.clone(),
                    Msg::ExpectedType(self.token.name()),
                ))
            }
        };

        let pos = ty.pos.clone();
        if self.token.is(TokenKind::LBracket) {
            self.advance_token()?;

            if self.token.is(TokenKind::RBracket) {
                self.advance_token()?;
                return Ok(Type::new(pos, TypeKind::Array(box ty, None)));
            } else {
                let len = if let TokenKind::LitInt(lit, _, _) = &self.token.kind {
                    lit.parse::<i64>().unwrap() as usize
                } else {
                    unimplemented!() // TODO: parse expression
                };
                self.advance_token()?;
                self.expect_token(TokenKind::RBracket)?;
                return Ok(Type::new(
                    pos,
                    TypeKind::Array(box ty.clone(), Some(len as _)),
                ));
            }
        }

        Ok(ty)
    }

    fn parse_unary(&mut self, opts: &ExprParsingOpts) -> ExprResult {
        match self.token.kind {
            TokenKind::Add | TokenKind::Sub | TokenKind::Not => {
                let tok = self.advance_token()?;
                let op = match tok.kind {
                    TokenKind::Add => "+",
                    TokenKind::Sub => "-",
                    TokenKind::Not => "!",
                    _ => unreachable!(),
                };

                let expr = self.parse_primary(opts)?;
                Ok(Box::new(Expr {
                    pos: tok.position,
                    id: self.generate_id(),
                    kind: ExprKind::Unary(op.to_owned(), expr),
                }))
            }
            TokenKind::Mul => {
                let pos = self.advance_token()?.position;
                let expr = self.parse_primary(opts)?;

                Ok(Box::new(Expr {
                    pos,
                    id: self.generate_id(),
                    kind: ExprKind::Deref(expr),
                }))
            }
            _ => self.parse_primary(opts),
        }
    }

    fn create_binary(&mut self, tok: Token, left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
        let op = match tok.kind {
            TokenKind::Eq => {
                return Box::new(Expr {
                    pos: tok.position,
                    id: self.generate_id(),
                    kind: ExprKind::Assign(left, right),
                });
            }

            TokenKind::Or => "||",
            TokenKind::And => "&&",
            TokenKind::EqEq => "==",
            TokenKind::Ne => "!=",
            TokenKind::Lt => "<",
            TokenKind::Le => "<=",
            TokenKind::Gt => ">",
            TokenKind::Ge => ">=",
            //TokenKind::EqEqEq => BinOp::Cmp(CmpOp::Is),
            //TokenKind::NeEqEq => BinOp::Cmp(CmpOp::IsNot),
            TokenKind::BitOr => "|",
            TokenKind::BitAnd => "&",
            TokenKind::Caret => "^",
            TokenKind::Add => "+",
            TokenKind::Sub => "-",
            TokenKind::Mul => "*",
            TokenKind::Div => "/",
            TokenKind::Mod => "%",
            TokenKind::LtLt => "<<",
            TokenKind::GtGt => ">>",
            TokenKind::GtGtGt => ">>>",
            _ => panic!("unimplemented token {:?}", tok),
        };

        Box::new(Expr {
            pos: tok.position,
            id: self.generate_id(),
            kind: ExprKind::Binary(op.to_owned(), left, right),
        })
    }

    fn parse_deref(&mut self) -> ExprResult {
        let pos = self.token.position.clone();
        self.expect_token(TokenKind::Mul)?;
        let expr = self.parse_expression()?;
        Ok(Box::new(Expr {
            pos,
            id: self.generate_id(),
            kind: ExprKind::Deref(expr),
        }))
    }

    fn parse_addrof(&mut self) -> ExprResult {
        let pos = self.token.position.clone();
        self.expect_token(TokenKind::BitAnd)?;
        let expr = self.parse_expression()?;
        Ok(Box::new(Expr {
            pos,
            id: self.generate_id(),
            kind: ExprKind::AddrOf(expr),
        }))
    }

    fn parse_parentheses(&mut self) -> ExprResult {
        self.advance_token()?;
        let exp = self.parse_expression()?;
        self.expect_token(TokenKind::RParen)?;

        Ok(exp)
    }

    fn parse_array_lit(&mut self) -> ExprResult {
        let pos = self.advance_token()?.position;

        let values = self.parse_comma_list(TokenKind::RBracket, |p| p.parse_expression())?;

        Ok(Box::new(Expr {
            pos: pos,
            id: self.generate_id(),
            kind: ExprKind::Array(values),
        }))
    }

    fn parse_factor(&mut self, opts: &ExprParsingOpts) -> ExprResult {
        match self.token.kind.clone() {
            TokenKind::BitAnd => self.parse_addrof(),
            TokenKind::LParen => self.parse_parentheses(),
            TokenKind::LBracket => self.parse_array_lit(),
            TokenKind::Mul => self.parse_deref(),
            TokenKind::LitChar(_) => self.parse_lit_char(),
            TokenKind::LitInt(_, _, _) => self.parse_lit_int(),
            TokenKind::LitFloat(_, _) => self.parse_lit_float(),
            TokenKind::String(_) => self.parse_string(),
            TokenKind::True | TokenKind::False => self.parse_bool_literal(),
            TokenKind::Null => self.parse_null(),
            TokenKind::SizeOf => self.parse_sizeof(),
            TokenKind::Identifier(_) => self.parse_identifier_or_call(opts),
            _ => Err(MsgWithPos::new(
                self.lexer.path().to_string(),
                self.src(),
                self.token.position.clone(),
                Msg::ExpectedFactor(self.token.name().clone()),
            )),
        }
    }
}

struct ExprParsingOpts {
    parse_struct_lit: bool,
}

impl ExprParsingOpts {
    pub const fn new() -> ExprParsingOpts {
        ExprParsingOpts {
            parse_struct_lit: true,
        }
    }

    pub fn parse_struct_lit(&mut self, val: bool) -> &mut ExprParsingOpts {
        self.parse_struct_lit = val;
        self
    }
}
