/// Корень программы — один или несколько классов
#[derive(Debug, Clone)]
pub struct Program {
    pub classes: Vec<ClassDecl>,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub methods: Vec<MethodDecl>,
    pub fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub vis: Visibility,
    pub is_static: bool,
    pub ty: Type,
    pub name: String,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub vis: Visibility,
    pub is_static: bool,
    pub return_ty: Type,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub ty: Type,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
}

/// Типы, поддерживаемые j2bit
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Int,
    Float,
    Bool,
    Str,
    Array(Box<Type>),
    Named(String), // пользовательский класс
}

/// Выражения
#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    BoolLit(bool),
    Ident(String),

    /// field.method / obj.field
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },

    /// obj.method(args)
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    /// Статический вызов: ClassName.method(args)
    StaticCall {
        class: String,
        method: String,
        args: Vec<Expr>,
    },

    /// new ClassName(args)
    New {
        class: String,
        args: Vec<Expr>,
    },

    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, LtEq, Gt, GtEq,
    And, Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Операторы
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Объявление локальной переменной: int x = 5;
    VarDecl {
        ty: Type,
        name: String,
        init: Option<Expr>,
    },

    Expr(Expr),

    Return(Option<Expr>),

    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },

    While {
        cond: Expr,
        body: Vec<Stmt>,
    },

    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Stmt>,
    },
}
