use listeners::ReceivesEvents;
use discord::model::Event;

#[allow(dead_code)]
pub struct EventDebugger;

impl ReceivesEvents for EventDebugger {
  fn receive(&self, event: &Event) {
    println!("{:#?}", event);
  }
}
