use std::sync;
use std::thread;

mod atomic_tx {
    use std::error;
    use std::fmt;
    use std::sync;

    pub(crate) type TxResult = Result<(), TxError>;

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

    pub(crate) trait TxOp {
        type TxState;
        fn apply(&self, state: &mut Self::TxState) -> TxResult;
        fn rollback(&self, state: &mut Self::TxState);
    }

    pub(crate) fn run<T>(ops: &[Box<dyn TxOp<TxState=T>>], data: &mut sync::Arc<sync::Mutex<T>>)
                         -> Result<(), (usize, TxError)> {
        let mut completed = 0;
        let mut err: Option<TxError> = None;
        let mut data = data.lock().unwrap();

        for op in ops {
            match op.apply(&mut *data) {
                Err(e) => {
                    err = Some(e);
                    break;
                }
                _ => completed += 1
            }
        }

        if err.is_some() {
            for op in ops[..completed].iter().rev() {
                op.rollback(&mut data);
            }
            return Err((completed, err.unwrap()));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct TxData {
    sender: u32,
    receiver: u32,
}

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

fn tx_fees(val: u32) -> u32 {
    match val {
        0...10 => 2,
        11...100 => 5,
        100...500 => 10,
        _ => val / 50
    }
}

fn main() {
    let data = sync::Arc::new(sync::Mutex::new(TxData { sender: 200, receiver: 305 }));
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
//            println!("{:?}", res);
        }).join();
    }
    println!("{:?}", data);
}
