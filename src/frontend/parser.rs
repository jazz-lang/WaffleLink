/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

use super::ast::*;
use super::lexer::*;
use super::msg::*;
use super::reader::Reader;
use super::token::*;
use std::mem;

pub struct Parser<'a> {
    lexer: Lexer,
    token: Token,
    ast: &'a mut Vec<Box<Expr>>,
}

macro_rules! expr {
    ($e:expr,$pos:expr) => {
        Box::new(Expr {
            pos: $pos,
            expr: $e,
        })
    };
}

type EResult = Result<Box<Expr>, MsgWithPos>;

impl<'a> Parser<'a> {
    pub fn new(reader: Reader, ast: &'a mut Vec<Box<Expr>>) -> Parser<'a> {
        Self {
            lexer: Lexer::new(reader),
            token: Token::new(TokenKind::End, Position::new(1, 1)),
            ast,
        }
    }

    fn init(&mut self) -> Result<(), MsgWithPos> {
        self.advance_token()?;

        Ok(())
    }

    pub fn parse(&mut self) -> Result<(), MsgWithPos> {
        self.init()?;
        while !self.token.is_eof() {
            self.parse_top_level()?;
        }
        Ok(())
    }

    fn expect_token(&mut self, kind: TokenKind) -> Result<Token, MsgWithPos> {
        if self.token.kind == kind {
            let token = self.advance_token()?;

            Ok(token)
        } else {
            Err(MsgWithPos::new(
                self.token.position,
                Msg::ExpectedToken(kind.name().into(), self.token.name()),
            ))
        }
    }

    fn parse_top_level(&mut self) -> Result<(), MsgWithPos> {
        let expr = self.parse_expression()?;

        self.ast.push(expr);
        Ok(())
    }

    fn parse_function(&mut self) -> EResult {
        let pos = self.expect_token(TokenKind::Fun)?.position;
        let name = if let TokenKind::Identifier(_) = &self.token.kind {
            Some(self.expect_identifier()?)
        } else {
            None
        };
        self.expect_token(TokenKind::LParen)?;
        /* let params = if self.token.kind == TokenKind::RParen {
            vec![]
        } else {
            let mut tmp = vec![];
            while !self.token.is(TokenKind::RParen) {
                tmp.push(self.expect_identifier()?);
                if !self.token.is(TokenKind::RParen) {
                    self.expect_token(TokenKind::Comma)?;
                }
            }
            tmp
        };
        self.expect_token(TokenKind::RParen)?;*/
        let params = self.parse_comma_list(TokenKind::RParen, |parser| parser.parse_arg())?;
        let block = self.parse_block()?;
        Ok(expr!(ExprKind::Function(name, params, block), pos))
    }

    fn parse_arg(&mut self) -> Result<Arg, MsgWithPos> {
        let pos = self.token.position;
        let tok = self.token.kind.name();
        match self.token.kind {
            TokenKind::Var => {
                self.advance_token()?;
                Ok(Arg::Ident(true, self.expect_identifier()?))
            }
            TokenKind::Identifier { .. } => Ok(Arg::Ident(false, self.expect_identifier()?)),
            TokenKind::LBrace => {
                self.expect_token(TokenKind::LBrace)?;
                let list: Vec<String> =
                    self.parse_comma_list(TokenKind::RBrace, |parser| parser.expect_identifier())?;
                Ok(Arg::Record(list))
            }
            TokenKind::LBracket => {
                self.expect_token(TokenKind::LBracket)?;
                let list: Vec<String> = self
                    .parse_comma_list(TokenKind::RBracket, |parser| parser.expect_identifier())?;
                Ok(Arg::Array(list))
            }
            _ => Err(MsgWithPos::new(
                pos,
                Msg::Custom(format!("unexpected token '{}' in argument position.", tok,)),
            )),
        }
    }

    fn parse_let(&mut self) -> EResult {
        let reassignable = self.token.is(TokenKind::Var);

        let pos = self.advance_token()?.position;
        let pat = self.parse_pattern()?;
        self.expect_token(TokenKind::Eq)?;
        let expr = self.parse_expression()?;
        Ok(expr!(ExprKind::Let(reassignable, pat, expr), pos))
    }

    fn parse_return(&mut self) -> EResult {
        let pos = self.expect_token(TokenKind::Return)?.position;
        let expr = self.parse_expression()?;
        Ok(expr!(ExprKind::Return(Some(expr)), pos))
    }

    fn parse_expression(&mut self) -> EResult {
        match self.token.kind {
            /*TokenKind::New => {
                let pos = self.advance_token()?.position;
                let calling = self.parse_expression()?;
                Ok(expr!(ExprKind::New(calling), pos))
            }*/
            TokenKind::Fun => self.parse_function(),
            TokenKind::Class => self.parse_class(),
            TokenKind::Match => self.parse_match(),
            TokenKind::Let | TokenKind::Var => self.parse_let(),
            TokenKind::LBrace => self.parse_block(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Throw => self.parse_throw(),
            TokenKind::Try => self.parse_try(),
            _ => self.parse_binary(0),
        }
    }

    fn parse_self(&mut self) -> EResult {
        let pos = self.expect_token(TokenKind::This)?.position;
        Ok(expr!(ExprKind::This, pos))
    }

    fn parse_throw(&mut self) -> EResult {
        let pos = self.token.position;
        self.advance_token()?;
        Ok(expr!(ExprKind::Throw(self.parse_expression()?), pos))
    }

    fn parse_try(&mut self) -> EResult {
        let pos = self.token.position;
        self.advance_token()?;
        let e = self.parse_expression()?;
        self.expect_token(TokenKind::Catch)?;
        let name = self.expect_identifier()?;
        let c = self.parse_expression()?;
        return Ok(expr!(ExprKind::Try(e,name,c),pos));
    }

    fn parse_while(&mut self) -> EResult {
        let pos = self.expect_token(TokenKind::While)?.position;
        let cond = self.parse_expression()?;
        let block = self.parse_block()?;
        Ok(expr!(ExprKind::While(cond, block), pos))
    }

    fn parse_if(&mut self) -> EResult {
        let pos = self.expect_token(TokenKind::If)?.position;
        let cond = self.parse_expression()?;
        let then_block = self.parse_expression()?;
        let else_block = if self.token.is(TokenKind::Else) {
            self.advance_token()?;

            if self.token.is(TokenKind::If) {
                let if_block = self.parse_if()?;
                let block = expr!(ExprKind::Block(vec![if_block]), if_block.pos);

                Some(block)
            } else {
                Some(self.parse_expression()?)
            }
        } else {
            None
        };

        Ok(expr!(ExprKind::If(cond, then_block, else_block), pos))
    }

    fn parse_block(&mut self) -> EResult {
        let pos = self.expect_token(TokenKind::LBrace)?.position;
        let mut exprs = vec![];
        while !self.token.is(TokenKind::RBrace) && !self.token.is_eof() {
            let expr = self.parse_expression()?;
            exprs.push(expr);
        }
        self.expect_token(TokenKind::RBrace)?;
        Ok(expr!(ExprKind::Block(exprs), pos))
    }

    fn create_binary(&mut self, tok: Token, left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
        let op = match tok.kind {
            TokenKind::Eq => return expr!(ExprKind::Assign(left, right), tok.position),
            TokenKind::Or => "||",
            TokenKind::And => "&&",
            TokenKind::BitOr => "|",
            TokenKind::BitAnd => "&",
            TokenKind::EqEq => "==",
            TokenKind::Ne => "!=",
            TokenKind::Lt => "<",
            TokenKind::Gt => ">",
            TokenKind::Le => "<=",
            TokenKind::Ge => ">=",
            TokenKind::Caret => "^",
            TokenKind::Add => "+",
            TokenKind::Sub => "-",
            TokenKind::Mul => "*",
            TokenKind::Div => "/",
            TokenKind::LtLt => "<<",
            TokenKind::GtGt => ">>",
            TokenKind::Mod => "%",
            _ => unimplemented!(),
        };

        expr!(ExprKind::BinOp(left, op.to_owned(), right), tok.position)
    }

    fn parse_binary(&mut self, precedence: u32) -> EResult {
        let mut left = self.parse_unary()?;
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
                TokenKind::BitOr | TokenKind::BitAnd | TokenKind::Caret => 6,
                TokenKind::LtLt | TokenKind::GtGt | TokenKind::Add | TokenKind::Sub => 8,
                TokenKind::Mul | TokenKind::Div | TokenKind::Mod => 9,
                _ => {
                    return Ok(left);
                }
            };
            if precedence >= right_precedence {
                return Ok(left);
            }

            let tok = self.advance_token()?;
            left = {
                let right = self.parse_binary(right_precedence)?;
                self.create_binary(tok, left, right)
            };
        }
    }

    pub fn parse_unary(&mut self) -> EResult {
        match self.token.kind {
            TokenKind::Add | TokenKind::Sub | TokenKind::Not => {
                let tok = self.advance_token()?;
                let op = match tok.kind {
                    TokenKind::Add => String::from("+"),
                    TokenKind::Sub => String::from("-"),
                    TokenKind::Not => String::from("!"),
                    _ => unreachable!(),
                };
                let expr = self.parse_primary()?;
                Ok(expr!(ExprKind::Unop(op, expr), tok.position))
            }
            _ => self.parse_primary(),
        }
    }

    /*pub fn parse_expression(&mut self) -> EResult {
        self.parse_binary(0)
    }*/

    pub fn parse_primary(&mut self) -> EResult {
        let mut left = self.parse_factor()?;
        loop {
            left = match self.token.kind {
                TokenKind::Dot => {
                    let tok = self.advance_token()?;
                    let ident = self.expect_identifier()?;
                    expr!(ExprKind::Access(left, ident), tok.position)
                }
                TokenKind::LBracket => {
                    let tok = self.advance_token()?;
                    let index = self.parse_expression()?;
                    self.expect_token(TokenKind::RBracket)?;
                    expr!(ExprKind::ArrayIndex(left, index), tok.position)
                }
                _ => {
                    if self.token.is(TokenKind::LParen) {
                        let expr = left;

                        self.expect_token(TokenKind::LParen)?;

                        let args =
                            self.parse_comma_list(TokenKind::RParen, |p| p.parse_expression())?;

                        expr!(ExprKind::Call(expr, args), expr.pos)
                    } else {
                        return Ok(left);
                    }
                }
            }
        }
    }

    fn expect_identifier(&mut self) -> Result<String, MsgWithPos> {
        let tok = self.advance_token()?;

        if let TokenKind::Identifier(ref value) = tok.kind {
            Ok(value.to_owned())
        } else {
            Err(MsgWithPos::new(
                tok.position,
                Msg::ExpectedIdentifier(tok.name()),
            ))
        }
    }

    fn parse_comma_list<F, R>(
        &mut self,
        stop: TokenKind,
        mut parse: F,
    ) -> Result<Vec<R>, MsgWithPos>
    where
        F: FnMut(&mut Parser) -> Result<R, MsgWithPos>,
    {
        let mut data = vec![];
        let mut comma = true;

        while !self.token.is(stop.clone()) && !self.token.is_eof() {
            if !comma {
                return Err(MsgWithPos::new(
                    self.token.position,
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

    fn parse_list<F, R>(&mut self, stop: TokenKind, mut parse: F) -> Result<Vec<R>, MsgWithPos>
    where
        F: FnMut(&mut Parser) -> Result<R, MsgWithPos>,
    {
        let mut data = vec![];

        while !self.token.is(stop.clone()) && !self.token.is_eof() {
            let entry = parse(self)?;
            data.push(entry);
        }

        self.expect_token(stop)?;

        Ok(data)
    }
    fn advance_token(&mut self) -> Result<Token, MsgWithPos> {
        let tok = self.lexer.read_token()?;

        Ok(mem::replace(&mut self.token, tok))
    }

    fn parse_lambda(&mut self) -> EResult {
        let tok = self.advance_token()?;
        let params = if tok.kind == TokenKind::Or {
            vec![]
        } else {
            self.parse_comma_list(TokenKind::BitOr, |f| f.parse_arg())?
        };

        let block = self.parse_expression()?;
        Ok(expr!(ExprKind::Lambda(params, block), tok.position))
    }
    fn parse_class(&mut self) -> EResult {
        let pos = self.expect_token(TokenKind::Class)?.position;
        let name = self.expect_identifier()?;
        let proto = if self.token.is(TokenKind::LParen) {
            self.advance_token()?;
            let proto = self.parse_expression()?;
            self.expect_token(TokenKind::RParen)?;
            Some(proto)
        } else {
            None
        };
        self.expect_token(TokenKind::LBrace)?;
        let body = self.parse_list(TokenKind::RBrace, |p| {
            let pos = p.token.position;
            let name = p.token.name().to_owned();
            let attr = match p.token.kind {
                TokenKind::Fun => p.parse_function()?,

                _ => return Err(MsgWithPos::new(pos, Msg::ExpectedClassElement(name))),
            };

            Ok(attr)
        })?;

        Ok(Expr {
            pos,
            expr: ExprKind::Class(name, proto, body),
        })
        .map(Box::new)
    }
    fn parse_match(&mut self) -> EResult {
        let pos = self.expect_token(TokenKind::Match)?.position;
        let e = self.parse_expression()?;
        self.expect_token(TokenKind::LBrace)?;
        let list = self.parse_comma_list(TokenKind::RBrace, |parser: &mut Parser| {
            let pat = parser.parse_pattern()?;
            let when_clause = if parser.token.is(TokenKind::When) || parser.token.is(TokenKind::Or)
            {
                parser.advance_token()?;
                Some(parser.parse_expression()?)
            } else {
                None
            };
            parser.expect_token(TokenKind::Arrow)?;
            let expr = parser.parse_expression()?;
            Ok((pat, when_clause, expr))
        })?;

        Ok(Expr {
            expr: ExprKind::Match(e, list),
            pos,
        })
        .map(|x| Box::new(x))
    }

    fn parse_pattern(&mut self) -> Result<Box<Pattern>, MsgWithPos> {
        let pos = self.token.position;
        match self.token.kind {
            TokenKind::Underscore => {
                self.advance_token()?;
                Ok(Box::new(Pattern {
                    pos,
                    decl: PatternDecl::Pass,
                }))
            }
            TokenKind::LitInt { .. } => self.plit_int(),
            TokenKind::LitFloat(_) => self.plit_float(),
            TokenKind::String(_) => self.plit_str(),
            TokenKind::Identifier(_) => self.pident(),
            TokenKind::LBracket => self.parray(),
            TokenKind::LBrace => self.precord(),

            TokenKind::DotDot => {
                let pos = self.advance_token()?.position;
                Ok(Pattern {
                    decl: PatternDecl::Rest,
                    pos: pos,
                })
                .map(|x| Box::new(x))
            }
            _ => unimplemented!(),
        }
    }

    pub fn parse_factor(&mut self) -> EResult {
        let expr = match self.token.kind {
            TokenKind::Fun => self.parse_function(),
            TokenKind::LParen => self.parse_parentheses(),
            TokenKind::LitChar(_) => self.lit_char(),
            TokenKind::LitInt(_, _, _) => self.lit_int(),
            TokenKind::LitFloat(_) => self.lit_float(),
            TokenKind::String(_) => self.lit_str(),
            TokenKind::Identifier(_) => self.ident(),
            TokenKind::This => self.parse_self(),
            TokenKind::BitOr | TokenKind::Or => self.parse_lambda(),
            TokenKind::True => self.parse_bool_literal(),
            TokenKind::False => self.parse_bool_literal(),
            TokenKind::Nil => self.parse_nil(),
            TokenKind::New => {
                let pos = self.token.position;
                self.advance_token()?;
                if self.token.is(TokenKind::LBrace) {
                    self.expect_token(TokenKind::LBrace)?;
                    let list = self.parse_comma_list(TokenKind::RBrace, |p| {
                        let name = p.expect_identifier()?;
                        let value = if p.token.is(TokenKind::Colon) {
                            p.advance_token()?;
                            Some(p.parse_expression()?)
                        } else {
                            None
                        };

                        Ok((name, value))
                    });
                    Ok(expr!(ExprKind::NewObject(list?), pos))
                } else {
                    let call = self.parse_expression()?;
                    if let ExprKind::Call { .. } = call.expr {
                        Ok(expr!(ExprKind::New(call), pos))
                    } else {
                        Err(MsgWithPos::new(
                            self.token.position,
                            Msg::Custom("Function call expected".to_owned()),
                        ))
                    }
                }
            }
            _ => Err(MsgWithPos::new(
                self.token.position,
                Msg::ExpectedFactor(self.token.name().clone()),
            )),
        };

        expr
    }

    fn parse_parentheses(&mut self) -> EResult {
        self.advance_token()?;
        let expr = self.parse_expression();
        self.expect_token(TokenKind::RParen)?;
        expr
    }

    fn parse_nil(&mut self) -> EResult {
        let tok = self.advance_token()?;
        let pos = tok.position;
        if let TokenKind::Nil = tok.kind {
            Ok(expr!(ExprKind::Nil, pos))
        } else {
            unreachable!()
        }
    }

    fn parse_bool_literal(&mut self) -> EResult {
        let tok = self.advance_token()?;
        let value = tok.is(TokenKind::True);
        Ok(expr!(ExprKind::ConstBool(value), tok.position))
    }

    fn lit_int(&mut self) -> EResult {
        let tok = self.advance_token()?;
        let pos = tok.position;
        if let TokenKind::LitInt(i, _, _) = tok.kind {
            Ok(expr!(ExprKind::ConstInt(i.parse().unwrap()), pos))
        } else {
            unreachable!()
        }
    }

    fn lit_char(&mut self) -> EResult {
        let tok = self.advance_token()?;
        let pos = tok.position;
        if let TokenKind::LitChar(c) = tok.kind {
            Ok(expr!(ExprKind::ConstChar(c), pos))
        } else {
            unreachable!()
        }
    }
    fn lit_float(&mut self) -> EResult {
        let tok = self.advance_token()?;
        let pos = tok.position;
        if let TokenKind::LitFloat(c) = tok.kind {
            Ok(expr!(ExprKind::ConstFloat(c.parse().unwrap()), pos))
        } else {
            unreachable!()
        }
    }
    fn lit_str(&mut self) -> EResult {
        let tok = self.advance_token()?;
        let pos = tok.position;
        if let TokenKind::String(s) = tok.kind {
            Ok(expr!(ExprKind::ConstStr(s), pos))
        } else {
            unreachable!()
        }
    }

    fn ident(&mut self) -> EResult {
        let pos = self.token.position;
        let ident = self.expect_identifier()?;

        Ok(expr!(ExprKind::Ident(ident), pos))
    }
    fn plit_int(&mut self) -> Result<Box<Pattern>, MsgWithPos> {
        let tok = self.advance_token()?;
        let pos = tok.position;
        if let TokenKind::LitInt(i, _, _) = tok.kind {
            Ok(Box::new(Pattern {
                decl: PatternDecl::ConstInt(i.parse::<i64>().unwrap()),
                pos,
            }))
        } else {
            unreachable!()
        }
    }

    fn plit_float(&mut self) -> Result<Box<Pattern>, MsgWithPos> {
        let tok = self.advance_token()?;
        let pos = tok.position;
        if let TokenKind::LitFloat(c) = tok.kind {
            Ok(Pattern {
                decl: PatternDecl::ConstFloat(c.parse().unwrap()),
                pos,
            })
            .map(|x| Box::new(x))
        } else {
            unreachable!()
        }
    }
    fn plit_str(&mut self) -> Result<Box<Pattern>, MsgWithPos> {
        let tok = self.advance_token()?;
        let pos = tok.position;
        if let TokenKind::String(s) = tok.kind {
            Ok(Pattern {
                decl: PatternDecl::ConstStr(s),
                pos,
            })
            .map(|x| Box::new(x))
        } else {
            unreachable!()
        }
    }

    fn pident(&mut self) -> Result<Box<Pattern>, MsgWithPos> {
        let pos = self.token.position;
        let ident = self.expect_identifier()?;

        Ok(Pattern {
            decl: PatternDecl::Ident(ident),
            pos: pos,
        })
        .map(|x| Box::new(x))
    }

    fn parray(&mut self) -> Result<Box<Pattern>, MsgWithPos> {
        let pos = self.token.position;
        self.expect_token(TokenKind::LBracket)?;
        let list = self.parse_comma_list(TokenKind::RBracket, |parser| parser.parse_pattern())?;

        Ok(Pattern {
            decl: PatternDecl::Array(list),
            pos,
        })
        .map(|x| Box::new(x))
    }

    fn precord(&mut self) -> Result<Box<Pattern>, MsgWithPos> {
        let pos = self.expect_token(TokenKind::LBrace)?.position;
        let record = self.parse_comma_list(TokenKind::RBrace, |parser| {
            let name = parser.expect_identifier()?;
            let pattern = if parser.token.is(TokenKind::Colon) {
                parser.expect_token(TokenKind::Colon)?;
                Some(parser.parse_pattern()?)
            } else {
                None
            };
            Ok((name, pattern))
        })?;

        Ok(Pattern {
            decl: PatternDecl::Record(record),
            pos,
        })
        .map(|x| Box::new(x))
    }
}
