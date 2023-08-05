mod pages;

use pages::home::Home;
use pages::not_found::NotFound;
use yew::prelude::*;
use yew_router::prelude::*;

pub struct App {}

pub enum Msg {}

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => {
            html! { <Home /> }
        }
        _ => {
            html! { <NotFound /> }
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        App {}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
            <div class="justify-content-center m-5">
                <div class="container-fluid .bg-image g-0" style="background-image: url('/data/images/buddhas.jpg'); height:325px;"/>
            </div>
            <main>
                <Switch<Route> render={Switch::render(switch)} />
            </main>
            </BrowserRouter>

        }
    }
}

pub fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    yew::start_app::<App>();
}
