#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate log;

use std::cell::RefCell;
use std::convert::TryInto;
use std::env;
use std::fmt::Debug;
use std::io::Error;
use std::mem::size_of;
use std::pin::Pin;
use std::ptr;
use std::rc::Rc; // single-threaded reference-counting pointer

use deno_core::CoreIsolate;
use deno_core::Op;
use deno_core::ResourceTable;
use deno_core::Script;
use deno_core::StartupData;
use deno_core::ZeroCopyBuf;

use futures::future::poll_fn;
use futures::prelude::*;
use futures::task::Context;
use futures::task::Poll;

use crate::state::State;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Record {
  pub promise_id: u32,
  pub rid: u32,
  pub result: i32,
}

type RecordBuf = [u8; size_of::<Record>()];

impl From<&[u8]> for Record {
  fn from(buf: &[u8]) -> Self {
    assert_eq!(buf.len(), size_of::<RecordBuf>());
    unsafe { *(buf as *const _ as *const RecordBuf) }.into()
  }
}

impl From<RecordBuf> for Record {
  fn from(buf: RecordBuf) -> Self {
    unsafe {
      #[allow(clippy::cast_ptr_alignment)]
      ptr::read_unaligned(&buf as *const _ as *const Self)
    }
  }
}

impl From<Record> for RecordBuf {
  fn from(record: Record) -> Self {
    unsafe { ptr::read(&record as *const _ as *const Self) }
  }
}

struct Isolate {
	core_isolate: Box<CoreIsolate>,
	state: State,
}

impl Isolate {
  pub fn new() -> Self {
    let startup_data = StartupData::Script(Script {
      source: include_str!("main.js"),
      filename: "main.js",
    });

    let mut isolate = Self {
      core_isolate: CoreIsolate::new(startup_data, false),
      state: Default::default(),
    };

    isolate.register_sync_op("run", op_run);
    isolate.register_sync_op("kill", op_kill);
    isolate.register_op("status", op_status);

    isolate
  }

  fn register_sync_op<F>(&mut self, name: &'static str, handler: F)
  where
    F: 'static + Fn(State, u32, Option<ZeroCopyBuf>) -> Result<u32, Error>,
  {
    let state = self.state.clone();
    let core_handler = move |_isolate: &mut CoreIsolate,
                             control_buf: &[u8],
                             zero_copy_buf: Option<ZeroCopyBuf>|
          -> Op {
      let state = state.clone();
      let record = Record::from(control_buf);
      let is_sync = record.promise_id == 0;
      assert!(is_sync);

      let result: i32 = match handler(state, record.rid, zero_copy_buf) {
        Ok(r) => r as i32,
        Err(_) => -1,
      };
      let buf = RecordBuf::from(Record { result, ..record })[..].into();
      Op::Sync(buf)
    };

    self.core_isolate.register_op(name, core_handler);
  }

  fn register_op<F>(
    &mut self,
    name: &'static str,
    handler: impl Fn(State, u32, Option<ZeroCopyBuf>) -> F + Copy + 'static,
  ) where
    F: TryFuture,
    F::Ok: TryInto<i32>,
    <F::Ok as TryInto<i32>>::Error: Debug,
  {
    let state = self.state.clone();
    let core_handler = move |_isolate: &mut CoreIsolate,
                             control_buf: &[u8],
                             zero_copy_buf: Option<ZeroCopyBuf>|
          -> Op {
      let state = state.clone();
      let record = Record::from(control_buf);
      let is_sync = record.promise_id == 0;
      assert!(!is_sync);

      let fut = async move {
        let op = handler(state, record.rid, zero_copy_buf);
        let result = op
          .map_ok(|r| r.try_into().expect("op result does not fit in i32"))
          .unwrap_or_else(|_| -1)
          .await;
        RecordBuf::from(Record { result, ..record })[..].into()
      };

      Op::Async(fut.boxed_local())
    };

    self.core_isolate.register_op(name, core_handler);
  }
}

impl Future for Isolate {
  type Output = <CoreIsolate as Future>::Output;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    self.core_isolate.poll_unpin(cx)
  }
}

fn op_run(
  state: State,
  _rid: u32,
  _buf: Option<ZeroCopyBuf>,
) -> Result<u32, Error> {
  debug!("run");
  Ok(0)
}

fn op_kill(
  state: State,
  _rid: u32,
  _buf: Option<ZeroCopyBuf>,
) -> Result<u32, Error> {
  debug!("kill");
  Ok(0)
}

fn op_status(
  state: State,
  rid: u32,
  _buf: Option<ZeroCopyBuf>,
) -> impl TryFuture<Ok = u32, Error = Error> {
  debug!("run status rid={}", rid);

  poll_fn(move |cx| {
  })
}

fn main() {
  // NOTE: `--help` arg will display V8 help and exit
  deno_core::v8_set_flags(env::args().collect());

  let isolate = Isolate::new();
}
