/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that sniffs data
use std::comm::{channel, Receiver, Sender};
use std::task::TaskBuilder;
use resource_task::{LoadData};

// use http::headers::content_type::MediaType;
// use http::headers::response::HeaderCollection as ResponseHeaderCollection;
// use http::headers::request::HeaderCollection as RequestHeaderCollection;
// use http::method::{Method, Get};
// use url::Url;

pub type SnifferTask = Sender<LoadData>;

pub fn new_sniffer_task() -> SnifferTask {
  let(sen, rec) = channel();
  let builder = TaskBuilder::new().named("SnifferManager");
  builder.spawn(proc(){
    SnifferManager::new(rec).start();
  });
}

struct SnifferManager {
  data_receiver: Receiver<LoadData>,
}

impl SnifferManager {
  fn new(data_receiver: Receiver <LoadData>) -> SnifferManager {
    SnifferManager {
      data_receiver: data_receiver,
    }
  }
}

impl SnifferManager {
  fn start(&self) {
    loop {
      let (load_data, start_chan) = self.data_receiver.recv();
      self.load(load_data, start_chan);

      // match self.data_receiver.recv() {
      //   Load(load_data, start_chan) => {
      //
      //   }
      //   Exit => {
      //     break
      //   }
      // }
    }
  }

  fn load(&self, load_data: LoadData, start_chan: Sender<LoadData>) {
    start_chan.send(load_data);
  }
}
