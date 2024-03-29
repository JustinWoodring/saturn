use anyhow::*;
use yew::{
	format::{Json, Nothing},
	prelude::*,
	services::fetch::{FetchService, FetchTask, Request, Response, StatusCode},
};

use crate::{
	components::{
		core::{router::*, *},
		ClubView,
		DetailsPage,
		NewClubPage,
		SearchBar,
	},
	tell,
	types::*,
};

pub enum Msg {
	FetchUserInfo,
	ReceieveUserInfo(AuthDetails),
	FailToReceiveUserInfo(Option<anyhow::Error>),
}

pub struct Home {
	link: ComponentLink<Self>,
	fetch_task: Option<FetchTask>,
	fetch_state: FetchState<AuthDetails>,
	details: Option<AuthDetails>,
	props: Props,
}

#[derive(Properties, Clone)]
pub struct Props {
	pub route: AppRoute,
}

impl Component for Home {
	type Message = Msg;
	type Properties = Props;

	fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
		Self {
			link,
			fetch_task: None,
			details: None,
			fetch_state: FetchState::Waiting,
			props,
		}
	}

	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		match msg {
			Msg::FetchUserInfo => {
				self.fetch_state = FetchState::Waiting;
				let req = Request::get("/api/auth/details").body(Nothing);

				match req {
					Ok(req) => {
						let callback = self.link.callback(
							|response: Response<Json<Result<AuthDetails, anyhow::Error>>>| {
								match response.status() {
									StatusCode::OK => {
										tell!("OK response");
										let Json(body) = response.into_body();

										match body {
											Ok(deets) => match deets.auth_level {
												AuthLevel::Guest => Msg::FailToReceiveUserInfo(
													Some(anyhow!("Guests must log in")),
												),
												_ => Msg::ReceieveUserInfo(deets),
											},
											Err(err) => Msg::FailToReceiveUserInfo(Some(err)),
										}
									}

									_ => Msg::FailToReceiveUserInfo(Some(anyhow!(
										"Weird status code received: {}",
										response.status()
									))),
								}
							},
						);

						match FetchService::fetch(req, callback) {
							Ok(task) => self.fetch_task = Some(task),
							Err(err) => tell!("{}", err),
						}
					}

					Err(err) => tell!("Error building request: {}", err),
				}
			}

			Msg::ReceieveUserInfo(data) => {
				self.fetch_state = FetchState::Done(data);
				self.fetch_task = None;
			}

			Msg::FailToReceiveUserInfo(maybe_error) => {
				self.fetch_task = None;

				if let Some(error) = maybe_error {
					tell!("Error getting user details: {:?}", error);
					self.fetch_state = FetchState::Failed(Some(error));
				} else {
					tell!("Unspecified error getting user details");
					self.fetch_state = FetchState::Failed(None);
				}
			}
		}

		true
	}

	fn change(&mut self, props: Self::Properties) -> ShouldRender {
		self.props = props;
		true
	}

	fn view(&self) -> Html {
		self.normal_view()
	}

	fn rendered(&mut self, first: bool) {
		if first {
			self.link.send_message(Msg::FetchUserInfo);
		}
	}
}

impl Home {
	fn normal_view(&self) -> Html {
		match &self.fetch_state {
			FetchState::Waiting => html! {
				<h1> {"Waiting..."} </h1>
			},

			FetchState::Done(details) => {
				html! {
					<div>
						<div id="root">
							<Toolbar pfp_url=details.picture.as_ref().unwrap().clone() username=format!("{} {}",details.first_name.clone().unwrap(), details.last_name.clone().unwrap())/>

							{
								match &self.props.route {
									AppRoute::Home => {
										html! {
											<ClubView/>
										}
									},

									AppRoute::Search => {
										html! {
											<SearchBar/>
										}
									},

									AppRoute::SearchTerm {term} => {
										html! {
											<SearchBar search_text=Some(term.to_owned())/>
										}
									},

									AppRoute::ClubForm => {
										html! {
											<NewClubPage/>
										}
									},

									AppRoute::Details{id} => {
										html! {
											<DetailsPage id=*id/>
										}
									}

									_ => unreachable!()
								}
							}
						</div>

						<Footer/>
					</div>
				}
			}

			FetchState::Failed(maybe_error) => {
				match maybe_error {
					Some(err) => tell!("Error: {:?}", err),
					None => tell!("Unspecified error occurred."),
				};

				html! {
					<AppRedirect route=AppRoute::Login/>
				}
			}
		}
	}
}
