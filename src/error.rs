#[derive(Debug)]
#[must_use]
pub struct ClickhouseUtilsError(
    String,
    Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
);

pub type Result<T> = std::result::Result<T, ClickhouseUtilsError>;

impl ClickhouseUtilsError {
    pub fn new(message: String) -> Self {
        ClickhouseUtilsError(message, None)
    }

    pub fn into_inner(self) -> Option<Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.1
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ClickhouseUtilsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClickhouseUtils error: {}", self.0)
    }
}

// We need to constrain the error to be Send + Sync + 'static to use it in eyre
impl std::error::Error for ClickhouseUtilsError where ClickhouseUtilsError: Send + Sync + 'static {}

impl From<std::io::Error> for ClickhouseUtilsError {
    fn from(error: std::io::Error) -> Self {
        ClickhouseUtilsError(format!("IO error: {}", error), Some(Box::new(error)))
    }
}

impl From<clickhouse::error::Error> for ClickhouseUtilsError {
    fn from(error: clickhouse::error::Error) -> Self {
        ClickhouseUtilsError(
            format!("Clickhouse error: {}", error),
            Some(Box::new(error)),
        )
    }
}

impl From<std::time::SystemTimeError> for ClickhouseUtilsError {
    fn from(error: std::time::SystemTimeError) -> Self {
        ClickhouseUtilsError(
            format!("System time error: {}", error),
            Some(Box::new(error)),
        )
    }
}
