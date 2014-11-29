/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that sniffs data
use std::comm::{channel, Receiver, Sender, Disconnected};
use std::task::TaskBuilder;
use resource_task::{TargetedLoadResponse, Payload, Done, LoadResponse};

pub type SnifferTask = Sender<TargetedLoadResponse>;

pub fn new_sniffer_task() -> SnifferTask {
    let(sen, rec) = channel();
    let builder = TaskBuilder::new().named("SnifferManager");
    builder.spawn(proc() {
        SnifferManager::new(rec).start();
    });
    sen
}

struct SnifferManager {
    data_receiver: Receiver<TargetedLoadResponse>,
}

impl SnifferManager {
    fn new(data_receiver: Receiver <TargetedLoadResponse>) -> SnifferManager {
        SnifferManager {
            data_receiver: data_receiver,
        }
    }
}

impl SnifferManager {
    fn start(self) {
        loop {
            match self.data_receiver.try_recv() {
                Ok(snif_data) => {
                    let mut resource_data = vec!();
                    loop {
                        match snif_data.load_response.progress_port.recv() {
                            Payload(data) => {
                                resource_data.push_all(data.as_slice());
                            }
                            Done(Ok(..)) => {
                                break;
                            }
                            Done(Err(..)) => {
                                break;
                            }
                        }
                    }
                    // We have all the data
                    // Do the sniffing
                    let (new_progress_chan, new_progress_port) = channel();

                    let load_response = LoadResponse {
                        progress_port: new_progress_port,
                        metadata: snif_data.load_response.metadata,
                    };
                    // replace metadata

                    let result = snif_data.consumer.send_opt(load_response);
                    if result.is_err() {
                        break;
                    }

                    new_progress_chan.send(Payload(resource_data));
                    new_progress_chan.send(Done(Ok(())));

                    // for x in resource_data.iter() {
                    //     println!("{}", x);
                    // }

                }
                Err(Disconnected) => break,
                Err(_) => (),
            }
        }
    }

}
