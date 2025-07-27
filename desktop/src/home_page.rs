use leptos::prelude::*;

use crate::{nav_bar::NavBar};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div>
            <h1>"Welcome to the Home Page"</h1>
            <p>"This is the home page of our application."</p>
        </div>
    }
}