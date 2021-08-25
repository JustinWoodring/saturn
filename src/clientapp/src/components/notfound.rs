use yew::prelude::*;

pub struct NotFound {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    props: Props,
}

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub route: Option<String>,
}

impl Component for NotFound {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h1> {Self.get_random_404_msg()} </h1>
            </div>
        }
    }
}

impl Component {
    fn get_random_404_msg() -> String {
        let responses = vec![
            "Are you lost?",
            "East? I thought you said \"weast...\""
        ];

        return responses[0];
    }
}