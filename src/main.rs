use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};
use std::thread;

type TxResult = Result<(), TxError>;

#[derive(Debug)]
struct TxError {
    description: &'static str
}

impl TxError {
    pub fn new(description: &'static str) -> Self {
        TxError { description }
    }
}

impl Error for TxError {
    fn description(&self) -> &str {
        self.description
    }
}

impl Display for TxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.description)
    }
}

trait TxOp {
    type TxState;
    fn execute(&self, state: &mut Self::TxState) -> TxResult;
    fn revert(&self, state: &mut Self::TxState);
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

impl TxOp for OpDebitSender {
    type TxState = TxData;

    fn execute(&self, state: &mut Self::TxState) -> TxResult {
        if state.sender < self.amount { return Err(TxError::new("Insufficient funds")); }
        state.sender = state.sender - self.amount;
        Ok(())
    }

    fn revert(&self, state: &mut Self::TxState) {
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

impl TxOp for OpCreditReceiver {
    type TxState = TxData;

    fn execute(&self, state: &mut Self::TxState) -> TxResult {
        state.receiver = state.receiver + self.amount;
        Ok(())
    }

    fn revert(&self, state: &mut Self::TxState) {
        state.receiver = state.receiver - self.amount;
    }
}

fn atomic_run(ops: &Vec<Box<dyn TxOp<TxState=TxData>>>, data: &mut Arc<Mutex<TxData>>)
              -> Result<usize, TxError> {
    let mut completed = 0;
    let mut data = data.lock().unwrap();
    for op in ops {
        op.execute(&mut *data)?;
        completed += 1;
    }
    if completed != ops.len() {
        for op in ops[..completed].iter().rev() {
            op.revert(&mut data);
        }
    }
    Ok(completed)
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
    let data = Arc::new(Mutex::new(TxData { sender: 200, receiver: 305 }));
    for i in 0..4 {
        let mut data = Arc::clone(&data);
        let _res = thread::spawn(move || {
            let transfer = 10 * i;
            let _res = atomic_run(&vec![
                Box::new(OpDebitSender::new(tx_fees(transfer))),
                Box::new(OpDebitSender::new(transfer)),
                Box::new(OpCreditReceiver::new(transfer)),
            ], &mut data);
            println!("{:?}", data);
        }).join();
    }
}
