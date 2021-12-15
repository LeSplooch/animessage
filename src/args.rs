use log::{debug, error, info, warn};
use thiserror::Error;

use smallvec::SmallVec;

use std::time::Duration;

type ArgsVec<'a> = SmallVec<[&'a str; 3]>;

pub(crate) struct Args<'a> {
    inner: ArgsVec<'a>,
}

impl<'a> Args<'a> {
    // TODO : create a nested enum in the not yet created 'animessage::Error' enum (in main.rs) for parsing errors and replace anyhow::Error with the corresponding variant
    pub(crate) fn parse(string_to_parse: &'a str, args_number_expected: usize) -> ArgsResult<Self> {
        let args = string_to_parse
            .split('"')
            .into_iter()
            .skip(1)
            .step_by(2)
            .collect::<ArgsVec>();

        let args_number_received = args.len();
        if args_number_received != args_number_expected {
            return Err(ArgsError::WrongArgsAmount {
                received: args_number_received,
                expected: args_number_expected,
            });
        }

        Ok(Args { inner: args })
    }

    // pub(crate) fn kwargs(&self, from_index: usize) -> ArgsResult<&[&'a str]> {
    //     let max_index = self.inner.len() - 1;
    //     if from_index > max_index {
    //         return Err(ArgsError::MissingArgs { index: from_index, max_index })
    //     }
    //     Ok(&self.inner[from_index..])
    // }

    pub(crate) fn get(&self, index: usize) -> &str {
        // let inner_len = &self.inner.len();
        // if index > *inner_len {
        //     return Err(ArgsError::MissingArgs { index: index, max_index: *inner_len })
        // }
        self.as_ref()[index]
    }
}

impl<'a> AsRef<ArgsVec<'a>> for Args<'a> {
    fn as_ref(&self) -> &ArgsVec<'a> {
        &self.inner
    }
}

impl<'a> AsMut<ArgsVec<'a>> for Args<'a> {
    fn as_mut(&mut self) -> &mut ArgsVec<'a> {
        &mut self.inner
    }
}

#[derive(Error, Debug)]
pub(crate) enum ArgsError {
    #[error("wrong number of arguments : received {received:?} arguments, but expected {expected:?} arguments.")]
    WrongArgsAmount { received: usize, expected: usize }, // (number of args received, number of args expected)
    #[error("check your function call for missing or misordered args. arg index {index:?} is out of bounds (max index : {max_index:?}). ")]
    MissingArgs { index: usize, max_index: usize },
}

pub(crate) type ArgsResult<T> = Result<T, ArgsError>;

pub(crate) fn duration_from_arg(duration: &str) -> anyhow::Result<Duration> { //
    match duration.parse::<f64>() {
        Ok(f) => {
            return Ok(Duration::from_secs_f64(f))
        },
        Err(_) => {
            anyhow::bail!("Can't convert the 1st argument into a decimal. Make sure your number is written as a decimal and not an integer. Example : Write 1.0 instead of 1.");
        },
    }
}
