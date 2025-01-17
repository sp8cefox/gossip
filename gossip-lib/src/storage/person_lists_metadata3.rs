use super::types::{PersonList1, PersonListMetadata3};
use crate::error::{Error, ErrorKind};
use crate::misc::Private;
use crate::storage::{RawDatabase, Storage};
use heed::types::Bytes;
use heed::RwTxn;
use speedy::{Readable, Writable};
use std::sync::Mutex;

// PersonList1 -> PersonListMetadata3 // bool is if private or not

static PERSON_LISTS_METADATA3_DB_CREATE_LOCK: Mutex<()> = Mutex::new(());
static mut PERSON_LISTS_METADATA3_DB: Option<RawDatabase> = None;

impl Storage {
    pub(super) fn db_person_lists_metadata3(&self) -> Result<RawDatabase, Error> {
        unsafe {
            if let Some(db) = PERSON_LISTS_METADATA3_DB {
                Ok(db)
            } else {
                // Lock.  This drops when anything returns.
                let _lock = PERSON_LISTS_METADATA3_DB_CREATE_LOCK.lock();

                // In case of a race, check again
                if let Some(db) = PERSON_LISTS_METADATA3_DB {
                    return Ok(db);
                }

                // Create it. We know that nobody else is doing this and that
                // it cannot happen twice.
                let mut txn = self.env.write_txn()?;
                let db = self
                    .env
                    .database_options()
                    .types::<Bytes, Bytes>()
                    // no .flags needed
                    .name("person_lists_metadata3")
                    .create(&mut txn)?;
                txn.commit()?;
                PERSON_LISTS_METADATA3_DB = Some(db);
                Ok(db)
            }
        }
    }

    pub(crate) fn get_person_list_metadata3(
        &self,
        list: PersonList1,
    ) -> Result<Option<PersonListMetadata3>, Error> {
        let key: Vec<u8> = list.write_to_vec()?;
        let txn = self.env.read_txn()?;
        Ok(match self.db_person_lists_metadata3()?.get(&txn, &key)? {
            None => None,
            Some(bytes) => {
                let mut plm = PersonListMetadata3::read_from_buffer(bytes)?;

                // Force followed list to be public
                if list == PersonList1::Followed {
                    plm.private = Private(false);
                }

                Some(plm)
            }
        })
    }

    pub(crate) fn set_person_list_metadata3<'a>(
        &'a self,
        list: PersonList1,
        metadata: &PersonListMetadata3,
        rw_txn: Option<&mut RwTxn<'a>>,
    ) -> Result<(), Error> {
        let key: Vec<u8> = list.write_to_vec()?;

        // Do not allow overwriting dtag or title of well defined lists:
        let bytes: Vec<u8> = if list == PersonList1::Muted {
            let mut md = metadata.to_owned();
            md.dtag = "muted".to_owned();
            md.title = "Muted".to_owned();
            md.write_to_vec()?
        } else if list == PersonList1::Followed {
            let mut md = metadata.to_owned();
            md.dtag = "followed".to_owned();
            md.title = "Followed".to_owned();
            md.private = Private(false);
            md.write_to_vec()?
        } else {
            metadata.write_to_vec()?
        };

        let mut local_txn = None;
        let txn = maybe_local_txn!(self, rw_txn, local_txn);

        self.db_person_lists_metadata3()?.put(txn, &key, &bytes)?;

        maybe_local_txn_commit!(local_txn);

        Ok(())
    }

    pub(crate) fn get_all_person_list_metadata3(
        &self,
    ) -> Result<Vec<(PersonList1, PersonListMetadata3)>, Error> {
        let txn = self.env.read_txn()?;
        let mut output: Vec<(PersonList1, PersonListMetadata3)> = Vec::new();
        for result in self.db_person_lists_metadata3()?.iter(&txn)? {
            let (key, val) = result?;
            let list = PersonList1::read_from_buffer(key)?;
            let mut metadata = PersonListMetadata3::read_from_buffer(val)?;

            // Force followed list to be public
            if list == PersonList1::Followed {
                metadata.private = Private(false);
            }

            output.push((list, metadata));
        }
        Ok(output)
    }

    pub(crate) fn find_person_list_by_dtag3(
        &self,
        dtag: &str,
    ) -> Result<Option<(PersonList1, PersonListMetadata3)>, Error> {
        let txn = self.env.read_txn()?;
        for result in self.db_person_lists_metadata3()?.iter(&txn)? {
            let (key, val) = result?;
            let list = PersonList1::read_from_buffer(key)?;
            let mut metadata = PersonListMetadata3::read_from_buffer(val)?;

            // Force followed list to be public
            if list == PersonList1::Followed {
                metadata.private = Private(false);
            }

            if metadata.dtag == dtag {
                return Ok(Some((list, metadata)));
            }
        }
        Ok(None)
    }

    pub(crate) fn allocate_person_list3<'a>(
        &'a self,
        metadata: &PersonListMetadata3,
        rw_txn: Option<&mut RwTxn<'a>>,
    ) -> Result<PersonList1, Error> {
        // Do not allocate for well-known names
        if &metadata.title == "Followed"
            || &metadata.title == "Muted"
            || &metadata.dtag == "followed"
            || &metadata.dtag == "muted"
        {
            return Err(ErrorKind::ListIsWellKnown.into());
        }

        // Check if it exists first (by dtag match)
        if let Some((found_list, _)) = self.find_person_list_by_dtag3(&metadata.dtag)? {
            return Err(ErrorKind::ListAlreadyExists(found_list).into());
        }

        let mut local_txn = None;
        let txn = maybe_local_txn!(self, rw_txn, local_txn);

        let mut slot: u8 = 0;

        for i in 2..=255 {
            let key: Vec<u8> = PersonList1::Custom(i).write_to_vec()?;
            if self.db_person_lists_metadata3()?.get(txn, &key)?.is_none() {
                slot = i;
                break;
            }
        }

        if slot < 2 {
            return Err(ErrorKind::ListAllocationFailed.into());
        }

        let list = PersonList1::Custom(slot);
        let key: Vec<u8> = list.write_to_vec()?;
        let val: Vec<u8> = metadata.write_to_vec()?;
        self.db_person_lists_metadata3()?.put(txn, &key, &val)?;

        maybe_local_txn_commit!(local_txn);

        Ok(list)
    }

    /// Deallocate this PersonList1
    pub(crate) fn deallocate_person_list3<'a>(
        &'a self,
        list: PersonList1,
        rw_txn: Option<&mut RwTxn<'a>>,
    ) -> Result<(), Error> {
        if u8::from(list) < 2 {
            return Err(ErrorKind::ListIsWellKnown.into());
        }

        let mut local_txn = None;
        let txn = maybe_local_txn!(self, rw_txn, local_txn);

        self.clear_person_list(list, Some(txn))?;

        // note: we dont have to delete the list of people because those
        //       lists are keyed by pubkey, and we already checked that
        //       this list is not referenced.
        let key: Vec<u8> = list.write_to_vec()?;
        self.db_person_lists_metadata3()?.delete(txn, &key)?;

        maybe_local_txn_commit!(local_txn);

        Ok(())
    }
}
