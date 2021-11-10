use std::fmt::Display;

use anyhow::anyhow;
use comrak::{ComrakExtensionOptions, ComrakOptions, arena_tree::Node, markdown_to_html};

use serde_json::{json};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Blob, FileReader, HtmlElement, HtmlImageElement, HtmlInputElement, HtmlTextAreaElement};
use yew::{Html, ShouldRender, agent::Dispatcher, format::{Bincode, Json, Nothing}, prelude::*, services::{
		fetch::{FetchTask, Request, Response, StatusCode},
		FetchService,
	}};

use crate::{components::{ClubCard, Spinner}, event::{Amogus, EventBus}, tell, types::{FetchState}};

pub struct NewClubPage {
	link: ComponentLink<Self>,
	club_name_field_contents: Option<String>,
	club_body_field_contents: Option<String>,
	long_club_description_contents: Option<String>,
	club_logo_preview_src: Option<String>,
	props: Props,
	post_task: Option<FetchTask>,
	post_logo_task: Option<FetchTask>,

	post_task_state: FetchState<()>,
	post_logo_task_state: FetchState<()>,
	img_selector_ref: NodeRef,
	img_preview_ref: NodeRef,
	markdown_preview_ref: NodeRef,
	club_name_input_ref: NodeRef,

	form_errors: Option<Vec<FormError>>,
	markdown_textarea_ref: NodeRef,
	toolbar_link: Dispatcher<EventBus>,

	right_col_ref: NodeRef,
	left_col_ref: NodeRef,
}

#[derive(Properties, Debug, Clone)]
pub struct Props {}

pub enum Msg {
	Ignore,
	UpdateInfoState(WhichTextField, String),
	UpdateClubLogoState(Vec<u8>),
	ValidateForm,
	PostClubLogo(i64),
	ReadLogo,

	PostClub,
	PostClubDone(i64),
	Reset
}

pub enum WhichTextField {
	TheNameOne,
	TheBodyOne,
	TheLongDescriptionOne,
}

#[derive(Debug)]
pub enum ReadImgResult {
	TooBig,
	EmptyFileList,
}

#[derive(Clone)]
pub enum FormError {
	ClubName(String),
	ClubDescription(String),
	ClubLogo(String),
}

impl FormError {
	fn unwrap(self) -> String {
		match self {
			Self::ClubName(string) => {
				string
			},

			Self::ClubDescription(string) => {
				string
			},

			Self::ClubLogo(string) => {
				string
			}
		}
	}
}

impl NewClubPage {
	fn reset(&mut self) {
		self.club_body_field_contents = None;
		self.club_name_field_contents = None;
		self.long_club_description_contents = None;
		self.post_task_state = FetchState::Waiting;
		self.post_task = None;
		if self.form_errors.is_some() {
			self.form_errors.take(); //Take and drop
		}

		self.img_preview_ref.cast::<HtmlImageElement>().unwrap().remove_attribute("src").unwrap();
		self.club_name_input_ref.cast::<HtmlInputElement>().unwrap().set_value("");
		self.markdown_textarea_ref.cast::<HtmlTextAreaElement>().unwrap().set_value("");

		self.markdown_preview_ref
			.cast::<HtmlElement>()
			.unwrap()
			.set_inner_html("");
	}

	fn add_form_error(&mut self, e: FormError) {
		if let Some(v) = self.form_errors.as_mut() {
			v.push(e);
		} else {
			self.form_errors = Some(vec![e]);
		}
	}

	fn read_img_sync(&self) {
		let file_reader = FileReader::new().expect("Unable to create file reader");
		let el = self.img_selector_ref.cast::<HtmlInputElement>().unwrap();

		if let Some(f) = el.files() {
			if let Some(file) = f.item(0) {
				let blob: &web_sys::Blob = file.as_ref();
				let link = self.link.clone();
				file_reader
					.read_as_data_url(&blob)
					.expect("Error reading image data");
				file_reader.set_onloadend(Some(
					Closure::once_into_js(move |x: ProgressEvent| {
						link.send_message(Msg::UpdateClubLogoState(
							x.target()
								.unwrap()
								.dyn_into::<FileReader>()
								.unwrap()
								.result()
								.unwrap()
								.as_string()
								.unwrap()
								.bytes()
								.collect(),
						))
					})
					.unchecked_ref(),
				));
			}
		}
	}

	fn get_form_errors(&self, e: FormError) -> Html {
		use FormError::*;
		let mut ret_errors: Vec<FormError> = vec![];

		if let Some(errors) = &self.form_errors {
			for error in errors {
				if let ClubLogo(_) = error {
					if let ClubLogo(_) = e {
						ret_errors.push(error.clone());
					}
				} 

				if let ClubName(_) = error {
					if let ClubName(_) = e {
						ret_errors.push(error.clone());
					}
				}

				if let ClubDescription(_) = error {
					if let ClubDescription(_) = e {
						ret_errors.push(error.clone());
					}
				}
			}

			html! {
				<>
					{
						for ret_errors.iter().map(|x| {
							html! {
								<h5>
									{
										x.clone().unwrap()
									}
								</h5>
							}
						})
					}
				</>
			}
		} else {
			html! {
				<>
				</>
			}
		}
	}
}

impl Component for NewClubPage {
	type Message = Msg;
	type Properties = Props;

	fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
		Self {
			link,
			club_name_field_contents: None,
			club_body_field_contents: None,
			long_club_description_contents: None,
			club_logo_preview_src: None,
			props,
			post_task: None,
			post_task_state: FetchState::Waiting,
			img_selector_ref: NodeRef::default(),
			img_preview_ref: NodeRef::default(),
			markdown_preview_ref: NodeRef::default(),
			club_name_input_ref: NodeRef::default(),
			post_logo_task: None,
			post_logo_task_state: FetchState::Waiting,
			form_errors: None,
			markdown_textarea_ref: NodeRef::default(),
			toolbar_link: Amogus::dispatcher(),
			left_col_ref: NodeRef::default(),
			right_col_ref: NodeRef::default(),
		}
	}

	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		match msg {
			Msg::Ignore => (),
			Msg::UpdateInfoState(which, value) => match which {
				WhichTextField::TheBodyOne => {
					self.club_body_field_contents = if value.len() > 0 { Some(value) } else { None }
				}

				WhichTextField::TheNameOne => {
					self.club_name_field_contents = if value.len() > 0 { Some(value) } else { None }
				}

				WhichTextField::TheLongDescriptionOne => {
					self.long_club_description_contents =
						if value.len() > 0 { Some(value) } else { None };
					let el = self.markdown_preview_ref.cast::<HtmlElement>().unwrap();

					el.set_inner_html(
						if let Some(md) = &self.long_club_description_contents {
							let md = markdown_to_html(
								md.as_str(),
								&ComrakOptions {
									extension: ComrakExtensionOptions {
										tagfilter: false,
										..ComrakExtensionOptions::default()
									},
									..ComrakOptions::default()
								},
							);

							ammonia::clean(md.as_str())
						} else {
							String::from("")
						}
						.as_str(),
					);
				}
			},

			Msg::ValidateForm => {
				let mut v = vec![];

				if self.club_name_field_contents.is_none() {
					v.push(FormError::ClubName("Club name cannot be empty.".to_owned()));
				}

				if self.long_club_description_contents.is_none() {
					v.push(FormError::ClubDescription("Club description cannot be empty.".to_owned()));
				}

				if let Some(el) = self.img_preview_ref.cast::<HtmlImageElement>() {

					let src = el.src();

					if src.is_empty() {
						v.push(FormError::ClubLogo("Club logo cannot be empty.".to_owned()));
					} else if src.bytes().len() > 1000000 {
						v.push(FormError::ClubLogo(format!("Club logo is too large. ({} MB > 1MB)", src.bytes().len() as f32 / 1000000.0)));
					}
				}

				if v.is_empty() {
					self.link.send_message(Msg::PostClub);
				} else {
					self.form_errors = Some(v);
				}
			}

			Msg::PostClub => {
				self.post_task_state = FetchState::Waiting;

				if let (Some(name), Some(body)) = (
					self.club_name_field_contents.clone(),
					self.long_club_description_contents.clone(),
				) {
					//FIXME back end often returns 422 on markdown with newlines and probably other stuff
					// Clean your body with ammonia
					let json = json!({"name": name, "body": ammonia::clean(&body)});

					let request = Request::post("http://localhost:443/api/clubs/create")
						.body(Json(&json))
						.unwrap();

					let response_callback = self.link.callback(
						|response: Response<Json<Result<String, anyhow::Error>>>| {
							match response.status() {
								StatusCode::OK => {
									tell!("Successfully post`ed club");
									tell!("{:?}", response);
									Msg::PostClubDone(20202)
								},

								_ => {
									tell!("Bad status receieved: {:?}", response.status());
									//Error stuff
									Msg::Ignore
								}
							}
						},
					);

					match FetchService::fetch(request, response_callback) {
						Ok(task) => self.post_task = Some(task),
						Err(err) => {
							tell!("Failed to post club: {}", err);
							self.post_task_state =
								FetchState::Failed(Some(anyhow!(format!("{:?}", err))));
						}
					}
				}
			}

			Msg::PostClubDone(id) => {
				self.reset();
				//self.link.send_message(Msg::PostClubLogo(id));
			}

			Msg::UpdateClubLogoState(bytes) => {
				let el = self.img_preview_ref.cast::<HtmlImageElement>().unwrap();

				match std::str::from_utf8(&bytes[0..]) {
					Ok(src) => el.set_src(src),
					Err(_) => {
						self.add_form_error(FormError::ClubLogo("Invalid image.".to_owned()))
					},
				}
			}

			Msg::PostClubLogo(id) => {}

			Msg::ReadLogo => {
				self.read_img_sync();
			},

			Msg::Reset => {
				if gloo_dialogs::confirm("Are you sure you want to clear all fields? This cannot be undone.") {
					self.reset();
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
		let club_name_field_cb = self.link.callback(|data: yew::html::InputData| {
			Msg::UpdateInfoState(WhichTextField::TheNameOne, data.value)
		});

		let description_cb = self.link.callback(|data: yew::html::InputData| {
			Msg::UpdateInfoState(WhichTextField::TheLongDescriptionOne, data.value)
		});

		let submit_cb = self.link.callback(|_| {
			Msg::ValidateForm
		});

		let reset_cb = self.link.callback(|_| {
			Msg::Reset
		});

		let image_input_callback = self
			.link
			.callback(|data: yew::html::InputData| Msg::ReadLogo);

		html! {
			<div class="new-club-page">
				<div ref=self.left_col_ref.clone() class="column col1">
					<h1>{"Create a new club!"}</h1>
					<div class="image-input">
						<img ref=self.img_preview_ref.clone() class="club-logo"/>
						<input ref=self.img_selector_ref.clone() oninput=image_input_callback type="file" name="file" id="file" class="inputfile" accept="image/png"/>
						<label for="file">{"Select a club logo"}</label>
						<small>{"(png files only. <= 1MB in size)"}</small>
					</div>
					<div class="form-errors">
						{
							self.get_form_errors(FormError::ClubLogo("".to_owned()))
						}
					</div>
					<input ref=self.club_name_input_ref.clone() oninput=club_name_field_cb class="club-input" type="text" placeholder="Club name"/>
					<div class="form-errors">
						{
							self.get_form_errors(FormError::ClubName("".to_owned()))
						}
					</div>
					<h3>{"Club description (markdown supported)"}</h3>
					<textarea ref=self.markdown_textarea_ref.clone() oninput=description_cb class="markdown-textarea"/>
					<div class="form-errors">
						{
							self.get_form_errors(FormError::ClubDescription("".to_owned()))
						}
					</div>
					<span>
						<button class="normal-button submit-new-club-button" onclick=submit_cb>{"Submit"}</button>
						{

							if self.post_task.is_some() {
								match self.post_task_state {
									FetchState::Waiting => html! {
										<>
											<h3>{"Submitting club..."}</h3>
											<Spinner/>
										</>
									},
									FetchState::Done(_) => html! {
										<>
											<h3>{"Done!"}</h3>
										</>
									},
									FetchState::Failed(_) => html! {
										<h3>{"Something bad happened."}</h3>
									},
								}
							} else {
								html! {
									<>
										<button class="normal-button destructive-button reset-club-page-button" onclick=reset_cb>{"Reset"}</button>
									</>
								}
							}
						}
					</span>
				</div>

				<div ref=self.right_col_ref.clone() class="column col2">
					<h1>{"Markdown preview"}</h1>
					<div ref=self.markdown_preview_ref.clone()>
					</div>
				</div>

			</div>
		}
	}

	fn rendered(&mut self, first: bool) {
		if first {
			self.club_name_input_ref.cast::<HtmlElement>().unwrap().focus().unwrap();
			self.left_col_ref.cast::<HtmlElement>().unwrap().class_list().add_1("new-club-page-col-in").unwrap();
			self.right_col_ref.cast::<HtmlElement>().unwrap().class_list().add_1("new-club-page-col-in").unwrap();
		}

		use crate::{components::core::toolbar::{Msg, WhichButton}, event::*};
		self.toolbar_link.send(Request::EventBusMsg(AgentMessage::ToolbarMsg(Msg::HighlightButton(WhichButton::AddClub))));
	}

	fn destroy(&mut self) {
		use crate::{components::core::toolbar::{Msg, WhichButton}, event::*};
		self.toolbar_link.send(Request::EventBusMsg(AgentMessage::ToolbarMsg(Msg::UnhighlightButton(WhichButton::AddClub))))
	}
}
