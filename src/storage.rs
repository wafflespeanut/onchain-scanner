use std::sync::Arc;

use redb::{Database, TableDefinition};

const IGNORED_ADDRS: TableDefinition<&str, ()> = TableDefinition::new("blacklist");

#[derive(Clone)]
pub struct Storage {
    db: Arc<Database>,
}

impl Storage {
    pub fn new(path: &str) -> shared::Result<Self> {
        Ok(Storage {
            db: Arc::new(Database::open(path)?),
        })
    }

    // TODO: redb has its own concurrency management, evaluate whether
    // it's blocking these methods

    pub async fn is_blocked(&self, contract_addr: &str) -> shared::Result<bool> {
        let exists;
        let tx = self.db.begin_read()?;
        {
            let table = tx.open_table(IGNORED_ADDRS)?;
            let res = table.get(contract_addr)?;
            exists = res.is_some();
        }
        Ok(exists)
    }

    pub async fn block_address(&self, contract_addr: &str) -> shared::Result<()> {
        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(IGNORED_ADDRS)?;
            table.insert(contract_addr, ())?;
        }
        tx.commit()?;
        Ok(())
    }

    pub async fn unblock_address(&self, contract_addr: &str) -> shared::Result<bool> {
        let exists;
        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(IGNORED_ADDRS)?;
            let res = table.remove(contract_addr)?;
            exists = res.is_some();
        }
        tx.commit()?;
        Ok(exists)
    }
}
