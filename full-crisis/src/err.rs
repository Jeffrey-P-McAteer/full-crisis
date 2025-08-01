pub type BoxError = Box<dyn std::error::Error + Send + 'static>;

#[derive(Debug)]
pub struct LocatedError {
    pub inner: Box<dyn std::error::Error>,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub addtl_msg: String,
}

// All this says is we assume the interior dyn Error is safe to access from not-the-constructing-thread.
unsafe impl Send for LocatedError {}

impl From<LocatedError> for Box<dyn std::error::Error + Send> {
    fn from(err: LocatedError) -> Self {
        Box::new(err)
    }
}

impl std::error::Error for LocatedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.inner)
    }
}

impl std::fmt::Display for LocatedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.addtl_msg.len() > 0 {
            write!(
                f,
                "{} from {}:{} ({})",
                self.inner, self.file, self.line, &self.addtl_msg
            )
        } else {
            write!(f, "{} from {}:{}", self.inner, self.file, self.line)
        }
    }
}

// The core idea: convenience macro to create the structure
#[macro_export]
macro_rules! eloc {
    () => {
        |e| full_crisis::err::LocatedError {
            inner: e.into(),
            file: file!(),
            line: line!(),
            column: column!(),
            addtl_msg: String::new(),
        }
    };
    ($msg:expr) => {
        |e| full_crisis::err::LocatedError {
            inner: e.into(),
            file: file!(),
            line: line!(),
            column: column!(),
            addtl_msg: $msg,
        }
    };
}

#[macro_export]
macro_rules! eloc_str {
    () => {
        |e| full_crisis::err::LocatedError {
            inner: format!("{:?}", e).into(),
            file: file!(),
            line: line!(),
            column: column!(),
            addtl_msg: String::new(),
        }
    };
    ($msg:expr) => {
        |e| full_crisis::err::LocatedError {
            inner: format!("{:?}", e).into(),
            file: file!(),
            line: line!(),
            column: column!(),
            addtl_msg: $msg,
        }
    };
}

pub use eloc;
pub use eloc_str;
