use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use yew::worker::*;

use crate::tell;

pub type Amogus = EventBus;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
	EventBusMsg(AgentMessage),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AgentMessage {
	ToolbarMsg(crate::components::core::toolbar::Msg),
	DetailsPageMsg(crate::components::clubs::pg_details::Msg),
	ClubFormMsg(crate::components::pg_new_club::Msg),
}

pub struct EventBus {
	link: AgentLink<EventBus>,
	subscribers: HashSet<HandlerId>,
}

impl Agent for EventBus {
	type Reach = Context<Self>;
	type Message = ();
	type Input = Request;
	type Output = AgentMessage;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			subscribers: HashSet::new(),
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::EventBusMsg(s) => {
				for sub in self.subscribers.iter() {
					tell!("{:?} sent {:?}", sub, s);
					self.link.respond(*sub, s.clone());
				}
			}
		}
	}

	fn connected(&mut self, id: HandlerId) {
		tell!("connecting id={:?}", id);
		self.subscribers.insert(id);
		tell!("inserted");
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);
	}
}
