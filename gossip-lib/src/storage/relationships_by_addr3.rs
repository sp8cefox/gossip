use crate::error::Error;
use crate::storage::types::RelationshipByAddr3;
use crate::storage::{RawDatabase, Storage};
use heed::RwTxn;
use heed::{types::Bytes, DatabaseFlags};
use nostr_types::{Id, NAddr};
use speedy::{Readable, Writable};
use std::sync::Mutex;

// Kind:Pubkey:d-tag -> RelationshipByAddr3:Id
//   (has dups)

static RELATIONSHIPS_BY_ADDR3_DB_CREATE_LOCK: Mutex<()> = Mutex::new(());
static mut RELATIONSHIPS_BY_ADDR3_DB: Option<RawDatabase> = None;

impl Storage {
    pub(super) fn db_relationships_by_addr3(&self) -> Result<RawDatabase, Error> {
        unsafe {
            if let Some(db) = RELATIONSHIPS_BY_ADDR3_DB {
                Ok(db)
            } else {
                // Lock.  This drops when anything returns.
                let _lock = RELATIONSHIPS_BY_ADDR3_DB_CREATE_LOCK.lock();

                // In case of a race, check again
                if let Some(db) = RELATIONSHIPS_BY_ADDR3_DB {
                    return Ok(db);
                }

                // Create it. We know that nobody else is doing this and that
                // it cannot happen twice.
                let mut txn = self.env.write_txn()?;
                let db = self
                    .env
                    .database_options()
                    .types::<Bytes, Bytes>()
                    .flags(DatabaseFlags::DUP_SORT) // NOT FIXED, RelationshipByAddr3 serialized isn't.
                    .name("relationships_by_addr3")
                    .create(&mut txn)?;
                txn.commit()?;
                RELATIONSHIPS_BY_ADDR3_DB = Some(db);
                Ok(db)
            }
        }
    }

    pub(crate) fn write_relationship_by_addr3<'a>(
        &'a self,
        addr: NAddr,
        related: Id,
        relationship_by_addr: RelationshipByAddr3,
        rw_txn: Option<&mut RwTxn<'a>>,
    ) -> Result<(), Error> {
        let key = relationships_by_addr3_into_key(&addr);
        let value = relationships_by_addr3_into_value(relationship_by_addr, related)?;

        let mut local_txn = None;
        let txn = maybe_local_txn!(self, rw_txn, local_txn);

        self.db_relationships_by_addr3()?.put(txn, &key, &value)?;

        maybe_local_txn_commit!(local_txn);

        Ok(())
    }

    pub(crate) fn find_relationships_by_addr3(
        &self,
        addr: &NAddr,
    ) -> Result<Vec<(Id, RelationshipByAddr3)>, Error> {
        let key = relationships_by_addr3_into_key(addr);
        let txn = self.env.read_txn()?;
        let iter = match self
            .db_relationships_by_addr3()?
            .get_duplicates(&txn, &key)?
        {
            Some(iter) => iter,
            None => return Ok(vec![]),
        };
        let mut output: Vec<(Id, RelationshipByAddr3)> = Vec::new();
        for result in iter {
            let (_key, val) = result?;
            let (rel, id) = relationships_by_addr3_from_value(val)?;
            output.push((id, rel));
        }
        Ok(output)
    }
}

fn relationships_by_addr3_into_key(ea: &NAddr) -> Vec<u8> {
    let u: u32 = ea.kind.into();
    let mut key: Vec<u8> = u.to_be_bytes().as_slice().to_owned();
    key.extend(ea.author.as_bytes());
    key.extend(ea.d.as_bytes());
    key.truncate(511);
    key
}

/*
fn relationships_by_addr3_from_key(key: &[u8]) -> Result<NAddr, Error> {
    let u = u32::from_be_bytes(key[..4].try_into().unwrap());
    let kind: EventKind = u.into();
    let pubkey: PublicKey = PublicKey::from_bytes(&key[4..4+32], true)?;
    let d: String = String::from_utf8_lossy(&key[4+32..]).to_string();
    Ok(NAddr {
        d,
        relays: vec![],
        kind,
        author: pubkey
    })
}
 */

fn relationships_by_addr3_into_value(
    relationship_by_addr: RelationshipByAddr3,
    id: Id,
) -> Result<Vec<u8>, Error> {
    let mut value: Vec<u8> = relationship_by_addr.write_to_vec()?;
    value.extend(id.as_slice());
    Ok(value)
}

fn relationships_by_addr3_from_value(value: &[u8]) -> Result<(RelationshipByAddr3, Id), Error> {
    let (result, len) = RelationshipByAddr3::read_with_length_from_buffer(value);
    let relationship_by_addr = result?;
    let id = Id(value[len..len + 32].try_into().unwrap());
    Ok((relationship_by_addr, id))
}
