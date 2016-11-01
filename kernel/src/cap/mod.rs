mod cpool;
mod untyped;
mod thread;
mod mapping;

pub use self::cpool::{CPoolHalf, CPoolFull, CPool, MDB, MDBAddr, CapFull, CapNearlyFull};
pub use self::untyped::{UntypedHalf, UntypedFull, UntypedNearlyFull};
pub use self::thread::{TCBHalf, TCBFull, TCB};
pub use abi::{CapSystemCall, CapSendMessage};
pub use arch::{TopPageTableHalf, TopPageTableFull, PageHalf, PageFull, ArchCap};

use common::*;
use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum Cap {
    CPool(CPoolFull),
    Untyped(UntypedFull),
    TCB(TCBFull),
    TopPageTable(TopPageTableFull),
    Page(PageFull),
    Arch(ArchCap),
}

impl Cap {
    pub unsafe fn set_mdb(&mut self, cpool: CPoolHalf, cpool_index: usize) {
        match self {
            &mut Cap::CPool(ref mut full) => full.set_mdb(cpool, cpool_index),
            &mut Cap::Untyped(ref mut full) => full.set_mdb(cpool, cpool_index),
            &mut Cap::TCB(ref mut full) => full.set_mdb(cpool, cpool_index),
            &mut Cap::TopPageTable(ref mut full) => full.set_mdb(cpool, cpool_index),
            &mut Cap::Page(ref mut full) => full.set_mdb(cpool, cpool_index),
            &mut Cap::Arch(ref mut full) => full.set_mdb(cpool, cpool_index),
        }
    }

    pub fn mdb(&self, index: usize) -> &MDB {
        match self {
            &Cap::CPool(ref full) => full.mdb(index),
            &Cap::Untyped(ref full) => full.mdb(index),
            &Cap::TCB(ref full) => full.mdb(index),
            &Cap::TopPageTable(ref full) => full.mdb(index),
            &Cap::Page(ref full) => full.mdb(index),
            &Cap::Arch(ref full) => full.mdb(index),
        }
    }

    pub fn mdb_mut(&mut self, index: usize) -> &mut MDB {
        match self {
            &mut Cap::CPool(ref mut full) => full.mdb_mut(index),
            &mut Cap::Untyped(ref mut full) => full.mdb_mut(index),
            &mut Cap::TCB(ref mut full) => full.mdb_mut(index),
            &mut Cap::TopPageTable(ref mut full) => full.mdb_mut(index),
            &mut Cap::Page(ref mut full) => full.mdb_mut(index),
            &mut Cap::Arch(ref mut full) => full.mdb_mut(index),
        }
    }
}

impl From<CPoolFull> for Cap { fn from(full: CPoolFull) -> Cap { Cap::CPool(full) } }
impl From<UntypedFull> for Cap { fn from(full: UntypedFull) -> Cap { Cap::Untyped(full) } }
impl From<TCBFull> for Cap { fn from(full: TCBFull) -> Cap { Cap::TCB(full) } }
impl From<TopPageTableFull> for Cap { fn from(full: TopPageTableFull) -> Cap { Cap::TopPageTable(full) } }
impl From<PageFull> for Cap { fn from(full: PageFull) -> Cap { Cap::Page(full) } }
impl From<ArchCap> for Cap { fn from(full: ArchCap) -> Cap { Cap::Arch(full) } }

pub trait SystemCallable {
    fn handle_send(&mut self, CapSendMessage);
}

pub trait CapReadObject<T, U: Deref<Target=T>> {
    fn read(&self) -> U;
}

pub trait CapReadRefObject<'a, T, U: Deref<Target=T> + 'a> {
    fn read(&'a self) -> U;
}

pub trait CapWriteObject<T, U: Deref<Target=T> + DerefMut> {
    fn write(&mut self) -> U;
}

pub trait CapWriteRefObject<'a, T, U: Deref<Target=T> + DerefMut + 'a> {
    fn write(&'a mut self) -> U;
}

// impl SystemCallable for Capability {
//     fn handle_send(&mut self, msg: CapSendMessage) {
//         match self {
//             &mut Capability::TCB(ref mut tcb) => {
//                 tcb.handle_send(msg);
//             },
//             _ => {
//                 log!("system call error: unhandled message");
//             }
//         }
//     }
// }
