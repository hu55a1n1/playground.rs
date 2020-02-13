use std::borrow::BorrowMut;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

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
        write!(f, "{:?}", self.description)
    }
}

trait TxOp {
    type TxState;
    fn execute(&mut self, state: &mut Self::TxState) -> Result<(), TxError>;
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

    fn execute(&mut self, state: &mut Self::TxState) -> Result<(), TxError> {
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

    fn execute(&mut self, state: &mut Self::TxState) -> Result<(), TxError> {
        state.receiver = state.receiver + self.amount;
        Ok(())
    }

    fn revert(&self, state: &mut Self::TxState) {
        state.receiver = state.receiver - self.amount;
    }
}

struct Tx<'a> {
    ops: Vec<Box<dyn TxOp<TxState=TxData>>>,
    data: &'a mut TxData,
    completed: usize,
}

impl<'a> Tx<'a> {
    pub fn run(ops: Vec<Box<dyn TxOp<TxState=TxData>>>, data: &'a mut TxData) -> Result<&mut TxData, TxError> {
        {
            let mut tx = Tx::new(ops, data);
            let res = tx.execute();
            if res.is_err() { return Err(res.err().unwrap()); }
        }
        return Ok(data);
    }

    fn new(ops: Vec<Box<dyn TxOp<TxState=TxData>>>, data: &'a mut TxData) -> Self {
        Tx { ops, completed: 0, data }
    }

    fn execute(&mut self) -> Result<(), TxError> {
        for op in &mut self.ops {
            op.execute(self.data)?;
            self.completed += 1;
        }
        return Ok(());
    }
}

impl<'a> Debug for Tx<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Tx {{ completed: {} }}", self.completed)
    }
}

impl<'a> Drop for Tx<'a> {
    fn drop(&mut self) {
        for op in &mut self.ops[self.completed..].iter().rev() {
            op.revert(&mut self.data);
        }
    }
}

fn main() {
    let mut data = TxData { sender: 10, receiver: 20 };
    let res = Tx::run(vec![Box::new(Op1::new())], data.borrow_mut());
    println!("{:?}", res.ok());
}
