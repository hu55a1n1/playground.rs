use std::fmt::{Debug, Error, Formatter};

trait TxOp {
    type TxState;
    fn execute(&mut self, state: &mut Self::TxState) -> Result<(), Error>;
    fn failed(&self) -> bool;
    fn revert(&self, state: &mut Self::TxState);
}

#[derive(Debug)]
struct TxData {
    sender: u32,
    receiver: u32,
}

struct Op1 {
    failed: bool
}

impl Op1 {
    pub fn new() -> Self {
        Op1 { failed: false }
    }
}

impl TxOp for Op1 {
    type TxState = TxData;

    fn execute(&mut self, state: &mut Self::TxState) -> Result<(), Error> {
        state.sender = state.sender - 10;
        self.failed = true;
        Ok(())
    }

    fn failed(&self) -> bool {
        return self.failed;
    }

    fn revert(&self, state: &mut Self::TxState) {
        if self.failed {
            println!("op1-rev");
            state.sender = state.sender + 10;
        }
    }
}

struct Tx {
    ops: Vec<Box<dyn TxOp<TxState=TxData>>>,
    data: TxData,
    completed: u8,
}

impl Tx {
    pub fn run(ops: Vec<Box<dyn TxOp<TxState=TxData>>>, data: TxData) -> Result<Tx, Error> {
        let mut tx = Tx::new(ops, data);
        let res = tx.execute();
        if res.is_err() { return Err(res.err().unwrap()); }
        return Ok(tx);
    }

    fn new(ops: Vec<Box<dyn TxOp<TxState=TxData>>>, data: TxData) -> Self {
        Tx { ops, data, completed: 0 }
    }

    fn execute(&mut self) -> Result<(), Error> {
        for op in &mut self.ops {
            op.execute(&mut self.data)?;
            self.completed += 1;
        }
        return Ok(());
    }
}

impl Debug for Tx {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Tx {{ completed: {}, data: {:?} }}", self.completed, self.data)
    }
}

impl Drop for Tx {
    fn drop(&mut self) {
        for op in &mut self.ops.iter().rev() {
            if op.failed() {
                op.revert(&mut self.data);
            }
        }
    }
}

fn main() {
    let data = TxData { sender: 10, receiver: 20 };
    let res = Tx::run(vec![Box::new(Op1::new())], data);
    println!("{:?}", res.ok());
}
