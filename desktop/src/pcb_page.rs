use leptos::{prelude::*, tachys::view, task::spawn_local};
use serde_wasm_bindgen::{from_value, to_value};
use shared::interface_types::{ClickCellArgs, Color, ColorGrid, MyResult, NewGridArgs};
use wasm_bindgen::prelude::*;

use crate::app::invoke;


#[component]
pub fn PcbPage() -> impl IntoView {
    let (err_msg, set_err_msg) = signal::<String>(String::new());
    // Reactive signals for rows and columns
    let (rows, set_rows) = signal::<usize>(10);
    let (cols, set_cols) = signal::<usize>(10);
    let (r, set_r) = signal::<u8>(0);
    let (g, set_g) = signal::<u8>(0);
    let (b, set_b) = signal::<u8>(0);
    fn create_new_grid(rows: usize, cols: usize)-> ColorGrid{
        let color_grid = (0..rows)
            .map(|_| {
                (0..cols)
                    .map(|_| Color {
                        r: 255,
                        g: 255,
                        b: 255,
                })
                .collect()
            })
            .collect();
        ColorGrid{grid: color_grid}
    }
    let (grid, set_grid) = signal::<ColorGrid>(create_new_grid(rows.get(), cols.get()));
    
    let new_grid = move || {
        spawn_local(async move{
            set_err_msg.set("fetching grid...".to_string());
            let args = NewGridArgs {
                rows: rows.get(),
                cols: cols.get(),
            };
            let args = to_value(&args).unwrap();
            let result = invoke("naive_new_grid", args).await;
            set_err_msg.set("grid fetched".to_string());
            let result = from_value::<MyResult<ColorGrid, String>>(result).unwrap();
            match result {
                MyResult::Ok(grid) => {                    
                    set_grid.set(grid);
                }
                MyResult::Err(err) => {
                    set_err_msg.set(err);
                }
            }
        });        
    };

    let on_cell_click = move |x: usize, y: usize| {
        // You can replace this with a Signal or any effect/handler you prefer
        set_err_msg.set(format!(
            "Clicked cell at ({}, {}) with color rgb({}, {}, {})",
            x, y, r.get(), g.get(), b.get()
        ));
        spawn_local(async move {
            set_err_msg.set("clicking cell...".to_string());
            let args = to_value(&ClickCellArgs {
                x,
                y,
                r: r.get(),
                g: g.get(),
                b: b.get(),
            })
            .unwrap();
            let result = invoke("naive_click_cell", args).await;
            let result = from_value::<MyResult<ColorGrid, String>>(result).unwrap();
            match result {
                MyResult::Ok(grid) => {
                    set_grid.set(grid);
                }
                MyResult::Err(err) => {
                    set_err_msg.set(err);
                }
            }
        });

    };
    view! {
        <div style="padding: 1rem;">
            <div>"Hello world"</div>
            <div style="margin-bottom: 1rem;">
            // rows, cols, new_grid, r, g, b, naive route, Bayesian route
                <label>"Rows: "</label>
                <input
                style="width: 3rem;"
                    type="number"
                    min="1"
                    prop:value=rows
                    on:input=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                            set_rows.set(val);
                        }
                    }
                />
                <label style="margin-left: 1rem;">"Cols: "</label>
                <input
                style="width: 3rem;"
                    type="number"
                    min="1"
                    prop:value=cols
                    on:input=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                            set_cols.set(val);
                        }
                    }
                />
                <button style="width: 6rem;" on:click=move |_| new_grid()>"New Grid"</button>
                <label>"r:"</label>
                <input
                style="width: 3rem;"
                    type="number"
                    min="0"
                    prop:value=r
                    on:input=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                            set_r.set(val);
                        }
                    }
                />
                <label>"g:"</label>
                <input
                style="width: 3rem;"
                    type="number"
                    min="0"
                    prop:value=g
                    on:input=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                            set_g.set(val);
                        }
                    }
                />
                <label>"b:"</label>
                <input
                style="width: 3rem;"
                    type="number"
                    min="0"
                    prop:value=b
                    on:input=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                            set_b.set(val);
                        }
                    }
                />
                <button style="width: 6rem;" on:click=move |_| {
                    set_err_msg.set("Routing...".to_string());
                    spawn_local(async move {                        
                        let result = invoke("naive_do_route", JsValue::NULL).await;
                        let result = from_value::<MyResult<ColorGrid, String>>(result).unwrap();
                        match result {
                            MyResult::Ok(grid) => {
                                set_grid.set(grid);
                                set_err_msg.set("Routing completed".to_string());
                            }
                            MyResult::Err(err) => {
                                set_err_msg.set(err);
                            }
                        }
                    });
                }>"Route"</button>
            </div>

            <div style="
            width: 600px;
            height: 400px;
            overflow: scroll;
            border: 1px solid #ccc;
            ">
            <div style="width: fit-content; height: fit-content;">
                {move ||{grid
                    .get()
                    .grid
                    .iter()
                    .enumerate()
                    .map(|(row_idx, row)| {
                        view! {
                            <div style="display: flex; flex-direction: row;">
                                {row.iter()
                                    .enumerate()
                                    .map(|(col_idx, Color{r, g, b})| {
                                        let color_str = format!("rgb({},{},{})", r, g, b);
                                        let y = row_idx;
                                        let x = col_idx;
                                        view! {
                                            <div style=format!(
                                                "width: 16px; height: 16px; background-color: {}; border: 2px solid black; display: inline",
                                                color_str,
                                            )
                                            on:click=move |_| on_cell_click(x, y)
                                            ></div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        }                        
                    })
                    .collect::<Vec<_>>()}
                }
                </div>
            </div>
        </div>
        <div style="color: rgb(255, 0, 0)">{err_msg}</div>
    }
}