use std::borrow::BorrowMut;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Mutex};

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

struct Tx<'a> {
    ops: Vec<Box<dyn TxOp<TxState=TxData>>>,
    data: &'a mut TxData,
    completed: usize,
}

impl<'a> Tx<'a> {
    pub fn run(ops: Vec<Box<dyn TxOp<TxState=TxData>>>, data: &'a mut Mutex<&'a mut TxData>)
               -> Result<&mut Mutex<&mut TxData>, TxError> {
        {
            let mut data1 = data.lock().unwrap();
            let mut tx = Tx::new(ops, data1.borrow_mut());
            let res = tx.execute();
            if res.is_err() { return Err(res.err().unwrap()); }
        }
        Ok(data)
    }

    fn new(ops: Vec<Box<dyn TxOp<TxState=TxData>>>, data: &'a mut TxData) -> Self {
        Tx { ops, completed: 0, data }
    }

    fn execute(&mut self) -> TxResult {
        for op in &mut self.ops {
            op.execute(self.data)?;
            self.completed += 1;
        }
        Ok(())
    }
}

impl<'a> Debug for Tx<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Tx {{ completed: {} }}", self.completed)
    }
}

impl<'a> Drop for Tx<'a> {
    fn drop(&mut self) {
        if self.completed != self.ops.len() {
            for op in &mut self.ops[..self.completed].iter().rev() {
                op.revert(&mut self.data);
            }
        }
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

fn tx_transfer(sender: u32, receiver: u32, transfer: u32) -> TxData {
    let mut data = TxData { sender, receiver };
    {
        let mut wrapped_data = Mutex::new(data.borrow_mut());
        let _res = Tx::run(vec![
            Box::new(OpDebitSender::new(tx_fees(transfer))),
            Box::new(OpDebitSender::new(transfer)),
            Box::new(OpCreditReceiver::new(transfer)),
        ], wrapped_data.borrow_mut());
    }
    data
}

fn main() {
    let data = tx_transfer(10, 20, 8);
    println!("{:?}", data);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_transfer_1() {
        let data = tx_transfer(10, 20, 8);
        assert_eq!(data.sender, 0);
        assert_eq!(data.receiver, 28);
    }

    #[test]
    fn tx_transfer_2() {
        let data = tx_transfer(120, 0, 35);
        assert_eq!(data.sender, 80);
        assert_eq!(data.receiver, 35);
    }

    #[test]
    fn tx_transfer_3() {
        let data = tx_transfer(912387, 31, 29387);
        assert_eq!(data.sender, 882413);
        assert_eq!(data.receiver, 29418);
    }

    #[test]
    fn tx_transfer_4() {
        let data = tx_transfer(0, 100, 0);
        assert_eq!(data.sender, 0);
        assert_eq!(data.receiver, 100);
    }

    #[test]
    fn tx_transfer_5() {
        let data = tx_transfer(80, 100, 80);
        assert_eq!(data.sender, 80);
        assert_eq!(data.receiver, 100);
    }
}