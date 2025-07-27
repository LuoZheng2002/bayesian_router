use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}


#[component]
pub fn App() -> impl IntoView {
    view! {
    <div>Hello world</div>
    <div id="root">
      // we wrap the whole app in a <Router/> to allow client-side navigation
      // from our nav links below      
      <Router>
        <NavBar/>
        <main>
          // <Routes/> both defines our routes and shows them on the page
          <Routes fallback=|| "Not found.">
              // users like /gbj or /bob
              <Route
                path=path!("/")
                view=HomePage
              />
              <Route
                path=path!("/naive")
                view=NaivePage
              />
              <Route
                path=path!("/proba")
                view=ProbaPage
              />
              // a fallback if the /:id segment is missing from the URL
              <Route
                path=path!("")
                view=move || view! { <p class="contact">"Select a contact."</p> }
              />
          </Routes>
        </main>
      </Router>
    </div>
  }
}
