use std::sync;
use std::thread;

///
/// Boiler-plate code written once
///
mod atomic_tx {
    use std::error;
    use std::fmt;
    use std::sync;

    pub(crate) type TxResult = Result<(), TxError>;

    /// Error type
    #[derive(Debug)]
    pub(crate) struct TxError {
        description: &'static str
    }

    impl TxError {
        pub fn new(description: &'static str) -> Self {
            TxError { description }
        }
    }

    impl error::Error for TxError {
        fn description(&self) -> &str {
            self.description
        }
    }

    impl fmt::Display for TxError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            write!(f, "{}", self.description)
        }
    }

    /// Operation interface
    /// Note: Rust supports `static polymorphism`!
    pub(crate) trait TxOp {
        // associated type
        type TxState;

        // applies this operation onto given state
        fn apply(&self, state: &mut Self::TxState) -> TxResult;

        // rollback this operation
        // *Must NOT fail*
        fn rollback(&self, state: &mut Self::TxState);
    }

    /// a function that applies the operations (specified by the ops array slice) one-by-one onto
    /// the shared state.
    /// Provides -
    /// 1. Reentrancy and atomicity
    /// 2. Strong exception guarantee - Either all ops are applied or none are applied and the
    /// shared state is exactly as before.
    pub(crate) fn run<T>(ops: &[Box<dyn TxOp<TxState=T>>], data: &mut sync::Arc<sync::Mutex<T>>)
                         -> Result<(), (usize, TxError)> {
        let mut completed = 0;
        let mut err: Option<TxError> = None;

        // `RAII` lock guard - guaranteed to unlock() when out-of-scope
        let mut data = data.lock().unwrap();

        // apply ops one-by-one, set err & break on failure
        for op in ops {
            match op.apply(&mut *data) {
                Err(e) => {
                    err = Some(e);
                    break;
                }
                _ => completed += 1
            }
        }

        // if err is set, rollback applied ops in reverse order
        if err.is_some() {
            for op in ops[..completed].iter().rev() {
                op.rollback(&mut data);
            }
            return Err((completed, err.unwrap()));
        }
        Ok(())
    }
}

///
/// Implementation code
///

/// Shared state
/// Maybe a database connection handle, ORM stuff, etc.
/// Here, we have two uints representative of sender & receiver account balances
#[derive(Debug)]
struct TxData {
    sender: u32,
    receiver: u32,
}

/// Debit sender operation
/// debits the sender account by specified amount
struct OpDebitSender {
    amount: u32
}

impl OpDebitSender {
    pub fn new(amount: u32) -> Self {
        OpDebitSender { amount }
    }
}

impl atomic_tx::TxOp for OpDebitSender {
    type TxState = TxData;

    fn apply(&self, state: &mut Self::TxState) -> atomic_tx::TxResult {
        if state.sender < self.amount { return Err(atomic_tx::TxError::new("Insufficient funds")); }
        state.sender = state.sender - self.amount;
        Ok(())
    }

    fn rollback(&self, state: &mut Self::TxState) {
        state.sender = state.sender + self.amount;
    }
}

/// Credit receiver operation
/// credits the receiver account with specified amount
struct OpCreditReceiver {
    amount: u32
}

impl OpCreditReceiver {
    pub fn new(amount: u32) -> Self {
        OpCreditReceiver { amount }
    }
}

impl atomic_tx::TxOp for OpCreditReceiver {
    type TxState = TxData;

    fn apply(&self, state: &mut Self::TxState) -> atomic_tx::TxResult {
        state.receiver = state.receiver + self.amount;
        Ok(())
    }

    fn rollback(&self, state: &mut Self::TxState) {
        state.receiver = state.receiver - self.amount;
    }
}

/// Tx fee calculation
fn tx_fees(val: u32) -> u32 {
    match val {
        0...10 => 2,
        11...100 => 5,
        100...500 => 10,
        _ => val / 50
    }
}

///
/// Main
///
fn main() {
    // shared state
    let data = sync::Arc::new(sync::Mutex::new(
        TxData { sender: 200, receiver: 305 }
    ));

    // usage example
    for i in 0..4 {
        let mut data = sync::Arc::clone(&data);
        let _res = thread::spawn(move || {
            let transfer = 10 * i;
            let ops: Vec<Box<dyn atomic_tx::TxOp<TxState=TxData>>> = vec![
                Box::new(OpDebitSender::new(tx_fees(transfer))),
                Box::new(OpDebitSender::new(transfer)),
                Box::new(OpCreditReceiver::new(transfer)),
            ];
            let _res = atomic_tx::run(&ops, &mut data);
        }).join();
    }
    println!("{:?}", data);
}
