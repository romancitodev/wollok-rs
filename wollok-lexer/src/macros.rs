use crate::token::Literal;

#[macro_export]
macro_rules! ident {
    (@expr $i:expr) => {
      $crate::token::Token::Ident($i)
    };
    (@raw $i:ident) => {
        String::from(stringify!($i))
    };
    (@raw $i:literal) => {
      String::from($i)
    };
    ($i:ident) => {
      $crate::token::Token::Ident(ident!(@raw $i))
    };
    ($i:literal) => {
      $crate::token::Token::Ident(ident!(@raw $i))
    };
}

#[macro_export]
macro_rules! lit {
    (Null) => {
        $crate::token::Token::Literal($crate::token::Literal::Null)
    };
    (@raw $i:expr) => {
        Into::<$crate::token::Token>::into($i)
    };
    ($i:expr) => {
        $crate::token::Token::Literal(Into::<$crate::token::Literal>::into($i))
    };
}

#[macro_export]
macro_rules! cmt {
    ($i:expr) => {
        $crate::token::Token::Comment($i.to_owned())
    };
}

#[macro_export]
macro_rules! T {
  (@raw $i:ident) => {
    $crate::token::Punctuation::$i
  };
  ($i:ident) => {
    $crate::token::Token::Punctuation(T!(@raw $i))
  }
}

#[macro_export]
macro_rules! kw {
  (@raw $i:ident) => {
    $crate::token::Keyword::$i
  };
  ($i:ident) => {
    $crate::token::Token::Keyword(kw!(@raw $i))
  };
  (@template $i:expr) => {
    $crate::token::Token::Keyword($crate::token::Keyword::Template($i))
  }
}

impl From<i64> for Literal {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Literal {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Literal {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}
