/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that sniffs data
use std::comm::{channel, Receiver, Sender};
use std::task::TaskBuilder;
use resource_task::{SnifferData};

pub enum SnifferControlMsg {
    Load(SnifferData),
    Exit
}

pub type SnifferTask = Sender<SnifferControlMsg>;

pub fn new_sniffer_task() -> SnifferTask {
  let(sen, rec) = channel();
  let builder = TaskBuilder::new().named("SnifferManager");
  builder.spawn(proc(){
    SnifferManager::new(rec).start();
  });
  sen
}

struct SnifferManager {
  data_receiver: Receiver<SnifferControlMsg>,
}

impl SnifferManager {
  fn new(data_receiver: Receiver <SnifferControlMsg>) -> SnifferManager {
    SnifferManager {
      data_receiver: data_receiver,
    }
  }
}

impl SnifferManager {
  fn start(&self) {
    loop {
      // case
      match self.data_receiver.recv() {
        Load(snif_data) => {
          self.load(snif_data);
        }
        Exit => {
          break
        }
      }
    }
  }

  fn load(&self, snif_data: SnifferData) {
    snif_data.tx.send(snif_data.load_data);
  }
}
