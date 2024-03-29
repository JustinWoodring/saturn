use yew::{html, Component, ComponentLink, Html, ShouldRender};

use crate::flags::IS_DEBUG_MODE;

pub struct GoogleLoginButton {
	link: ComponentLink<Self>,
	url: String,
}

pub enum Msg {
	GooglePLLoaded,
}

impl Component for GoogleLoginButton {
	type Message = Msg;
	type Properties = ();

	fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
		Self {
			link,
			url: String::from(if *IS_DEBUG_MODE {
				"http://localhost:443/api/auth/login"
			} else {
				"https://joinsaturn.net/api/auth/login"
			}),
		}
	}

	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		match msg {
			Msg::GooglePLLoaded => true,
			_ => false,
		}
	}

	fn change(&mut self, _props: Self::Properties) -> ShouldRender {
		false
	}

	fn view(&self) -> Html {
		html! {
			<div>
				<script id="gplib_script" src="https://accounts.google.com/gsi/client"></script>

				<div id="google-button-container">
					<div id="g_id_onload"
						data-client_id="699719776672-56jqfpk1g2uq8tma72hi56n5jkan82nr.apps.googleusercontent.com"
						data-login_uri=self.url.clone()
						data-ux_mode="redirect"
						data-auto_prompt="false">
					</div>
					<div class="g_id_signin"
							data-size="large"
							data-theme="outline"
							data-text="sign_in_with"
							data-shape="square"
							data-logo_alignment="left">
					</div>
				</div>
			</div>
		}
	}
}
