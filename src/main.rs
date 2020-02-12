use std::borrow::BorrowMut;
use std::fmt::{Debug, Error, Formatter};

trait TxOp {
    type TxState;
    fn execute(&mut self, state: &mut Self::TxState) -> Result<(), Error>;
    fn revert(&self, state: &mut Self::TxState);
}

#[derive(Debug)]
struct TxData {
    sender: u32,
    receiver: u32,
}

struct Op1;

impl Op1 {
    pub fn new() -> Self {
        Op1 {}
    }
}

impl TxOp for Op1 {
    type TxState = TxData;

    fn execute(&mut self, state: &mut Self::TxState) -> Result<(), Error> {
        state.sender = state.sender - 10;
        Ok(())
    }

    fn revert(&self, state: &mut Self::TxState) {
        println!("op1-rev");
        state.sender = state.sender + 10;
    }
}

struct Tx<'a> {
    ops: Vec<Box<dyn TxOp<TxState=TxData>>>,
    data: &'a mut TxData,
    completed: usize,
}

impl<'a> Tx<'a> {
    pub fn run(ops: Vec<Box<dyn TxOp<TxState=TxData>>>, data: &'a mut TxData) -> Result<&mut TxData, Error> {
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

    fn execute(&mut self) -> Result<(), Error> {
        for op in &mut self.ops {
            op.execute(self.data)?;
            self.completed += 1;
        }
        return Ok(());
    }
}

impl<'a> Debug for Tx<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
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
