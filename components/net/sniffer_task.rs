/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that sniffs data
use std::comm::{channel, Receiver, Sender};
use std::task::TaskBuilder;
use resource_task::{LoadResponse};

pub type SnifferTask = Sender<LoadResponse>;

pub fn new_sniffer_task(next_rx: Sender<LoadResponse>) -> SnifferTask {
  let(sen, rec) = channel();
  let builder = TaskBuilder::new().named("SnifferManager");
  builder.spawn(proc(){
    SnifferManager::new(rec).start(next_rx);
  });
  sen
}

struct SnifferManager {
  data_receiver: Receiver<LoadResponse>,
}

impl SnifferManager {
  fn new(data_receiver: Receiver <LoadResponse>) -> SnifferManager {
    SnifferManager {
      data_receiver: data_receiver,
    }
  }
}

impl SnifferManager {
  fn start(&self, next_rx: Sender<LoadResponse>) {
    loop {
      self.load(next_rx.clone(), self.data_receiver.recv());
    }
  }

  fn load(&self, next_rx: Sender<LoadResponse>, snif_data: LoadResponse) {
    next_rx.send(snif_data);
  }
}
