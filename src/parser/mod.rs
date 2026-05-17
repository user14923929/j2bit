use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use anyhow::{bail, Result};

pub fn parse(tokens: Vec<Token>) -> Result<Program> {
    let mut p = Parser::new(tokens);
    p.parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_token(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<()> {
        if self.peek() == kind {
            self.advance();
            Ok(())
        } else {
            let t = self.peek_token();
            bail!(
                "expected {:?}, got {:?} at line {}, col {}",
                kind,
                self.peek(),
                t.line,
                t.col
            )
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.advance();
                Ok(name)
            }
            other => {
                let t = self.peek_token();
                bail!(
                    "expected identifier, got {:?} at line {}, col {}",
                    other,
                    t.line,
                    t.col
                )
            }
        }
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.peek() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    // ── Program ──────────────────────────────────────────────

    fn parse_program(&mut self) -> Result<Program> {
        let mut classes = Vec::new();
        while self.peek() != &TokenKind::Eof {
            classes.push(self.parse_class()?);
        }
        Ok(Program { classes })
    }

    // ── Class ────────────────────────────────────────────────

    fn parse_class(&mut self) -> Result<ClassDecl> {
        // Опциональный `public`
        self.eat(&TokenKind::Public);
        self.expect(&TokenKind::Class)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;

        let mut methods = Vec::new();
        let mut fields = Vec::new();

        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            let vis = if self.eat(&TokenKind::Public) {
                Visibility::Public
            } else if self.eat(&TokenKind::Private) {
                Visibility::Private
            } else {
                Visibility::Public
            };
            let is_static = self.eat(&TokenKind::Static);
            let ty = self.parse_type()?;
            let member_name = self.expect_ident()?;

            if self.peek() == &TokenKind::LParen {
                // Метод
                methods.push(self.parse_method_rest(vis, is_static, ty, member_name)?);
            } else {
                // Поле
                let init = if self.eat(&TokenKind::Eq) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(&TokenKind::Semicolon)?;
                fields.push(FieldDecl {
                    vis,
                    is_static,
                    ty,
                    name: member_name,
                    init,
                });
            }
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(ClassDecl {
            name,
            methods,
            fields,
        })
    }

    fn parse_method_rest(
        &mut self,
        vis: Visibility,
        is_static: bool,
        return_ty: Type,
        name: String,
    ) -> Result<MethodDecl> {
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(MethodDecl {
            vis,
            is_static,
            return_ty,
            name,
            params,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        if self.peek() == &TokenKind::RParen {
            return Ok(params);
        }
        loop {
            let ty = self.parse_type()?;
            let name = self.expect_ident()?;
            params.push(Param { ty, name });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        Ok(params)
    }

    // ── Types ────────────────────────────────────────────────

    fn parse_type(&mut self) -> Result<Type> {
        let ty = match self.peek().clone() {
            TokenKind::Void => {
                self.advance();
                Type::Void
            }
            TokenKind::Int => {
                self.advance();
                Type::Int
            }
            TokenKind::Float => {
                self.advance();
                Type::Float
            }
            TokenKind::Bool => {
                self.advance();
                Type::Bool
            }
            TokenKind::StringType => {
                self.advance();
                Type::Str
            }
            TokenKind::Ident(n) => {
                self.advance();
                Type::Named(n)
            }
            other => bail!(
                "expected type, got {:?} at line {}",
                other,
                self.peek_token().line
            ),
        };
        // массив?
        if self.eat(&TokenKind::LBracket) {
            self.expect(&TokenKind::RBracket)?;
            return Ok(Type::Array(Box::new(ty)));
        }
        Ok(ty)
    }

    // ── Statements ───────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek().clone() {
            TokenKind::Return => {
                self.advance();
                let val = if self.peek() == &TokenKind::Semicolon {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.expect(&TokenKind::Semicolon)?;
                Ok(Stmt::Return(val))
            }

            TokenKind::If => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                self.expect(&TokenKind::LBrace)?;
                let then_body = self.parse_block()?;
                self.expect(&TokenKind::RBrace)?;
                let else_body = if self.eat(&TokenKind::Else) {
                    self.expect(&TokenKind::LBrace)?;
                    let b = self.parse_block()?;
                    self.expect(&TokenKind::RBrace)?;
                    Some(b)
                } else {
                    None
                };
                Ok(Stmt::If {
                    cond,
                    then_body,
                    else_body,
                })
            }

            TokenKind::While => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                self.expect(&TokenKind::LBrace)?;
                let body = self.parse_block()?;
                self.expect(&TokenKind::RBrace)?;
                Ok(Stmt::While { cond, body })
            }

            // Объявление переменной: int x = ...
            TokenKind::Int | TokenKind::Float | TokenKind::Bool | TokenKind::StringType => {
                let ty = self.parse_type()?;
                let name = self.expect_ident()?;
                let init = if self.eat(&TokenKind::Eq) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(&TokenKind::Semicolon)?;
                Ok(Stmt::VarDecl { ty, name, init })
            }

            _ => {
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::Semicolon)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    // ── Expressions ──────────────────────────────────────────

    fn parse_expr(&mut self) -> Result<Expr> {
        let lhs = self.parse_or()?;
        if self.eat(&TokenKind::Eq) {
            let rhs = self.parse_expr()?;
            return Ok(Expr::Assign {
                target: Box::new(lhs),
                value: Box::new(rhs),
            });
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut lhs = self.parse_and()?;
        while self.eat(&TokenKind::Or) {
            let rhs = self.parse_and()?;
            lhs = Expr::BinOp {
                op: BinOp::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut lhs = self.parse_cmp()?;
        while self.eat(&TokenKind::And) {
            let rhs = self.parse_cmp()?;
            lhs = Expr::BinOp {
                op: BinOp::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_cmp(&mut self) -> Result<Expr> {
        let lhs = self.parse_add()?;
        let op = match self.peek() {
            TokenKind::EqEq => BinOp::Eq,
            TokenKind::NotEq => BinOp::NotEq,
            TokenKind::Lt => BinOp::Lt,
            TokenKind::LtEq => BinOp::LtEq,
            TokenKind::Gt => BinOp::Gt,
            TokenKind::GtEq => BinOp::GtEq,
            _ => return Ok(lhs),
        };
        self.advance();
        let rhs = self.parse_add()?;
        Ok(Expr::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn parse_add(&mut self) -> Result<Expr> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_mul()?;
            lhs = Expr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> Result<Expr> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_unary()?;
            lhs = Expr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.eat(&TokenKind::Minus) {
            let e = self.parse_postfix()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Neg,
                expr: Box::new(e),
            });
        }
        if self.eat(&TokenKind::Not) {
            let e = self.parse_postfix()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Not,
                expr: Box::new(e),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            if !self.eat(&TokenKind::Dot) {
                break;
            }
            let field = self.expect_ident()?;
            if self.peek() == &TokenKind::LParen {
                self.advance();
                let args = self.parse_args()?;
                self.expect(&TokenKind::RParen)?;
                expr = Expr::MethodCall {
                    object: Box::new(expr),
                    method: field,
                    args,
                };
            } else {
                expr = Expr::FieldAccess {
                    object: Box::new(expr),
                    field,
                };
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.peek().clone() {
            TokenKind::IntLiteral(n) => {
                self.advance();
                Ok(Expr::IntLit(n))
            }
            TokenKind::FloatLiteral(f) => {
                self.advance();
                Ok(Expr::FloatLit(f))
            }
            TokenKind::StringLiteral(s) => {
                self.advance();
                Ok(Expr::StrLit(s))
            }
            TokenKind::BoolLiteral(b) => {
                self.advance();
                Ok(Expr::BoolLit(b))
            }

            TokenKind::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(e)
            }

            TokenKind::New => {
                self.advance();
                let class = self.expect_ident()?;
                self.expect(&TokenKind::LParen)?;
                let args = self.parse_args()?;
                self.expect(&TokenKind::RParen)?;
                Ok(Expr::New { class, args })
            }

            TokenKind::Ident(name) => {
                self.advance();
                // Статический вызов: ClassName.method(...)
                if self.peek() == &TokenKind::Dot {
                    // Смотрим вперёд — следующий за точкой идентификатор,
                    // а за ним — `(`, тогда StaticCall
                    if self.pos + 2 < self.tokens.len() {
                        if let TokenKind::Ident(_) = &self.tokens[self.pos + 1].kind {
                            if self.tokens[self.pos + 2].kind == TokenKind::LParen {
                                self.advance(); // .
                                let method = self.expect_ident()?;
                                self.advance(); // (
                                let args = self.parse_args()?;
                                self.expect(&TokenKind::RParen)?;
                                return Ok(Expr::StaticCall {
                                    class: name,
                                    method,
                                    args,
                                });
                            }
                        }
                    }
                }
                Ok(Expr::Ident(name))
            }

            other => {
                let t = self.peek_token();
                bail!(
                    "unexpected token {:?} in expression at line {}, col {}",
                    other,
                    t.line,
                    t.col
                )
            }
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        if self.peek() == &TokenKind::RParen {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        Ok(args)
    }
}
