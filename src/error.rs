use crate::impl_into_u8;
use cosmwasm_std::StdError;
use schemars::{JsonSchema, _serde_json::to_string};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug, JsonSchema)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    OverflowOccurred,
    TxNotConfirmationReady,
    TxAlreadyCancelled,
    TxAlreadyCompleted,
    TxNotConfirmed,
    TxNotRecurring,
    InvalidStartTime,
    InvalidEndTime
}

impl_into_u8!(Error);

impl CodeType for Error {
    fn to_verbose(&self, context: &Vec<&str>) -> String {
        match self{
            Error::OverflowOccurred => {
                build_string("Overflow error occurred. Check values", context)
            }
            Error::TxNotConfirmationReady => {
                build_string("Tx is not at confirmation state. Current state is '{}'", context)
            }
            Error::TxAlreadyCancelled => {
                build_string("Tx at position {} has already been cancelled", context)
            }
            Error::TxAlreadyCompleted => {
                build_string("Tx at position {} has already been completed", context)
            }
            Error::TxNotConfirmed => {
                build_string("Tx is not confirmed and ready to be fulfilled. Current state is '{}'", context)
            }
            Error::TxNotRecurring => {
                build_string("Tx selected isn't recurring", context)
            }
            Error::InvalidStartTime => {
                build_string("Start time of {} must be before selected end time of {}", context)
            }
            Error::InvalidEndTime => {
                build_string("End time of {} must be after current time of {}", context)
            }
        }
    }
}

#[macro_export]
macro_rules! impl_into_u8 {
    ($error:ident) => {
        impl From<$error> for u8 {
            fn from(err: $error) -> u8 {
                err as _
            }
        }
    };
}

const SILK_PAY_TARGET: &str = "silk_pay";

pub fn overflow_occurred() -> StdError {
    DetailedError::from_code(SILK_PAY_TARGET, Error::OverflowOccurred, vec![]).to_error()
}

pub fn tx_not_at_confirmation_stage(status: u8) -> StdError {
    let mut current_state = "";
    match status {
        1 => current_state = "Receiver Confirmed Address",
        2 => current_state = "Tx Cancelled",
        3 => current_state = "Tx Completed",
        5 => current_state = "Receiver Confirmed Address, Recurring Tx Active",
        _ => current_state = "Error Misfire"
    }
    DetailedError::from_code(SILK_PAY_TARGET, Error::TxNotConfirmationReady, vec![current_state]).to_error()
}

pub fn tx_already_cancelled(position: u32) -> StdError {
    let pos_string = position.to_string();
    let pos_str = &pos_string;
    DetailedError::from_code(SILK_PAY_TARGET, Error::TxAlreadyCancelled, vec![pos_str]).to_error()
}

pub fn tx_already_completed(position: u32) -> StdError {
    let pos_string = position.to_string();
    let pos_str = &pos_string;
    DetailedError::from_code(SILK_PAY_TARGET, Error::TxAlreadyCompleted, vec![pos_str]).to_error()
}

pub fn tx_not_confirmed(status: u8) -> StdError {
    let mut current_state = "";
    match status {
        0 => current_state = "Tx Unconfirmed",
        2 => current_state = "Tx Cancelled",
        3 => current_state = "Tx Completed",
        4 => current_state = "Tx Unconfirmed",
        _ => current_state = "Error Misfire"
    }
    DetailedError::from_code(SILK_PAY_TARGET, Error::TxNotConfirmationReady, vec![current_state]).to_error()
}

pub fn tx_not_recurring() -> StdError {
    DetailedError::from_code(SILK_PAY_TARGET, Error::TxNotRecurring, vec![]).to_error()
}

pub fn invalid_start_time(start: u64, end: u64) -> StdError {
    let start_string = start.to_string();
    let start_str = &start_string;
    let end_string = end.to_string();
    let end_str = &end_string;
    DetailedError::from_code(SILK_PAY_TARGET, Error::InvalidStartTime, vec![start_str, end_str]).to_error()
}

pub fn invalid_end_time(end: u64, now: u64, config_end_time_limit: u64) -> StdError {
    let now_string = now.to_string();
    let now_str = &now_string;
    let end_string = end.to_string();
    let end_str = &end_string;
    let config_string = config_end_time_limit.to_string();
    let config_str = &config_string;
    DetailedError::from_code(SILK_PAY_TARGET, Error::InvalidStartTime, vec![end_str, now_str]).to_error()
}
/**
 * ======================================================================================================
 * Error function setups past here, this divide is for user readability
 */
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DetailedError<T: CodeType> {
    pub target: String,
    pub code: u8,
    pub r#type: T,
    pub context: Vec<String>,
    pub verbose: String,
}

impl<T: CodeType + Serialize> DetailedError<T> {
    pub fn to_error(&self) -> StdError {
        StdError::generic_err(self.to_string())
    }

    pub fn to_string(&self) -> String {
        to_string(&self).unwrap_or("".to_string())
    }

    pub fn from_code(target: &str, code: T, context: Vec<&str>) -> Self {
        let verbose = code.to_verbose(&context);
        Self { target: target.to_string(), code: code.to_code(), r#type: code, context: context.iter().map(|s| s.to_string()).collect(), verbose }
    }
}

pub fn build_string(verbose: &str, context: &Vec<&str>) -> String {
    let mut msg = verbose.to_string();
    for arg in context.iter() {
        msg = msg.replacen("{}", arg, 1);
    }
    msg
}

pub trait CodeType: Into<u8> + Clone {
    fn to_code(&self) -> u8 {
        self.clone().into()
    }
    fn to_verbose(&self, context: &Vec<&str>) -> String;
}

#[cfg(test)]
pub mod tests {
    use crate::error::{build_string, CodeType, DetailedError};
    use cosmwasm_std::StdError;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug, JsonSchema)]
    #[repr(u8)]
    #[serde(rename_all = "snake_case")]
    enum TestCode {
        Error1,
        Error2,
        Error3,
    }

    impl_into_u8!(TestCode);

    impl CodeType for TestCode {
        fn to_verbose(&self, context: &Vec<&str>) -> String {
            match self {
                TestCode::Error1 => build_string("Error", context),
                TestCode::Error2 => build_string("Broke in {}", context),
                TestCode::Error3 => build_string("Expecting {} but got {}", context),
            }
        }
    }

    // Because of set variables, you could implement something like this

    fn error_1() -> StdError {
        DetailedError::from_code("contract", TestCode::Error1, vec![]).to_error()
    }

    fn error_2(context: &[&str; 1]) -> StdError {
        DetailedError::from_code("contract", TestCode::Error2, context.to_vec()).to_error()
    }

    fn error_3(context: &[&str; 2]) -> StdError {
        DetailedError::from_code("contract", TestCode::Error3, context.to_vec()).to_error()
    }

    #[test]
    fn string_builder() {
        assert_eq!(
            build_string("Test string {}", &vec!["arg"]),
            "Test string arg".to_string()
        )
    }

    #[test]
    fn to_code() {
        let code1 = TestCode::Error1;
        assert_eq!(code1.to_code(), 0);

        let code2 = TestCode::Error2;
        assert_eq!(code2.to_code(), 1);

        let code3 = TestCode::Error3;
        assert_eq!(code3.to_code(), 2);
    }

    #[test]
    fn to_verbose() {
        assert_eq!(TestCode::Error1.to_verbose(&vec![]), "Error".to_string());
        assert_eq!(
            TestCode::Error2.to_verbose(&vec!["function"]),
            "Broke in function".to_string()
        );
        assert_eq!(
            TestCode::Error3.to_verbose(&vec!["address", "amount"]),
            "Expecting address but got amount".to_string()
        );
    }

    #[test]
    fn from_code() {
        let err1 = DetailedError::from_code("contract", TestCode::Error1, vec![]);
        assert_eq!(err1.code, 0);
        assert_eq!(err1.r#type, TestCode::Error1);
        let empty: Vec<String> = vec![];
        assert_eq!(err1.context, empty);
        assert_eq!(err1.verbose, "Error".to_string());

        let err2 = DetailedError::from_code("contract", TestCode::Error2, vec!["function"]);
        assert_eq!(err2.code, 1);
        assert_eq!(err2.r#type, TestCode::Error2);
        assert_eq!(err2.context, vec!["function".to_string()]);
        assert_eq!(err2.verbose, "Broke in function".to_string());

        let err3 =
            DetailedError::from_code("contract", TestCode::Error3, vec!["address", "amount"]);
        assert_eq!(err3.code, 2);
        assert_eq!(err3.r#type, TestCode::Error3);
        assert_eq!(err3.context, vec![
            "address".to_string(),
            "amount".to_string()
        ]);
        assert_eq!(err3.verbose, "Expecting address but got amount".to_string());
    }

    #[test]
    fn to_string() {
        assert_eq!(DetailedError::from_code("contract", TestCode::Error1, vec![]).to_string(),
                   "{\"target\":\"contract\",\"code\":0,\"type\":\"error1\",\"context\":[],\"verbose\":\"Error\"}".to_string());
        assert_eq!(DetailedError::from_code("contract", TestCode::Error2, vec!["function"]).to_string(),
                   "{\"target\":\"contract\",\"code\":1,\"type\":\"error2\",\"context\":[\"function\"],\"verbose\":\"Broke in function\"}".to_string());
        assert_eq!(DetailedError::from_code("contract", TestCode::Error3, vec!["address", "amount"]).to_string(),
                   "{\"target\":\"contract\",\"code\":2,\"type\":\"error3\",\"context\":[\"address\",\"amount\"],\"verbose\":\"Expecting address but got amount\"}".to_string());
    }

    #[test]
    fn to_error() {
        let err1 = DetailedError::from_code("contract", TestCode::Error1, vec![]).to_error();
        match err1 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":0,\"type\":\"error1\",\"context\":[],\"verbose\":\"Error\"}".to_string()),
            _ => assert!(false)
        }

        let err2 =
            DetailedError::from_code("contract", TestCode::Error2, vec!["function"]).to_error();
        match err2 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":1,\"type\":\"error2\",\"context\":[\"function\"],\"verbose\":\"Broke in function\"}".to_string()),
            _ => assert!(false)
        }

        let err3 =
            DetailedError::from_code("contract", TestCode::Error3, vec!["address", "amount"])
                .to_error();
        match err3 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":2,\"type\":\"error3\",\"context\":[\"address\",\"amount\"],\"verbose\":\"Expecting address but got amount\"}".to_string()),
            _ => assert!(false)
        }
    }

    #[test]
    fn helpers() {
        let err1 = error_1();
        match err1 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":0,\"type\":\"error1\",\"context\":[],\"verbose\":\"Error\"}".to_string()),
                _ => assert!(false)
        }

        let err2 = error_2(&["function"]);
        match err2 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":1,\"type\":\"error2\",\"context\":[\"function\"],\"verbose\":\"Broke in function\"}".to_string()),
            _ => assert!(false)
        }

        let err3 = error_3(&["address", "amount"]);
        match err3 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":2,\"type\":\"error3\",\"context\":[\"address\",\"amount\"],\"verbose\":\"Expecting address but got amount\"}".to_string()),
            _ => assert!(false)
        }
    }
}
