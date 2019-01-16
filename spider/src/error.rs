use failure::Fail;
use libxml::parser::XmlParseError;

#[derive(Debug, Fail)]
pub enum MyError {
    #[fail(display = "XML parse error: got null pointer")]
    XmlParse,
    #[fail(display = "failed to create xpath context")]
    ContextInit,
    #[fail(display = "failed to evaluate xpath")]
    XPathEval,
    #[fail(display = "http error")]
    HttpError,
}

impl From<XmlParseError> for MyError {
    fn from(_: XmlParseError) -> MyError {
        MyError::XmlParse
    }
}

pub type MyResult<T> = Result<T, MyError>;
