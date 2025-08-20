use crate::token::Literal;

#[macro_export]
#[doc(hidden)]
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

#[doc(inline)]
pub use ident;

#[macro_export]
#[doc(hidden)]
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

#[doc(inline)]
pub use lit;

#[macro_export]
#[doc(hidden)]
macro_rules! cmt {
    ($i:expr) => {
        $crate::token::Token::Comment($i.to_owned())
    };
}

#[doc(inline)]
pub use cmt;

#[macro_export]
#[doc(hidden)]
macro_rules! T {
  (@raw $i:ident) => {
    $crate::token::Punctuation::$i
  };
  ($i:ident) => {
    $crate::token::Token::Punctuation(T!(@raw $i))
  }
}

#[doc(inline)]
pub use T;

#[macro_export]
#[doc(hidden)]
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

#[doc(inline)]
pub use kw;

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
