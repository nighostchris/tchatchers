// Copyright ⓒ 2022 LABEYE Loïc
// This tool is distributed under the MIT License, check out [here](https://github.com/nag763/tchatchers/blob/main/LICENSE.MD).

pub mod message_rmenu;
pub mod profile_rmenu;

use std::rc::Rc;

use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::Closure, JsCast};
use yew::{html, Component, Context, Html, Properties};
use yew_agent::{Bridge, Bridged, Dispatched};
use yew_router::scope_ext::{LocationHandle, RouterScopeExt};

use crate::services::rmenu_bus::{RMenuBus, RMenusBusEvents};

use self::{
    message_rmenu::{MessageRMenu, MessageRMenuProps},
    profile_rmenu::{ProfileRMenu, ProfileRMenuProps},
};

const JS_EVENTS_TO_CANCEL: [&str; 2] = ["click", "contextmenu"];

fn close_context_menu_function() -> Function {
    let closure: Box<dyn FnMut(_)> = Box::new(move |_: web_sys::MouseEvent| {
        RMenuBus::dispatcher().send(RMenusBusEvents::CloseRMenu)
    });

    let closure = Closure::wrap(closure);

    let js_value = closure.into_js_value();

    js_value.unchecked_into::<Function>()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum RMenuKind {
    MessageRMenu(MessageRMenuProps),
    ProfileRMenu(ProfileRMenuProps),
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {}

pub enum Msg {
    HandleBusMessage(RMenusBusEvents),
}

pub struct RightMenu {
    visible: bool,
    pos_x: i32,
    pos_y: i32,
    _producer: Box<dyn Bridge<RMenuBus>>,
    close_events: Vec<Function>,
    _location_handle: LocationHandle,
    content: Html,
}

impl Component for RightMenu {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let cb = {
            let link = ctx.link().clone();
            move |mc| {
                link.send_message(Msg::HandleBusMessage(mc));
            }
        };
        let listener = ctx
            .link()
            .add_location_listener(
                ctx.link()
                    .callback(|_e| Msg::HandleBusMessage(RMenusBusEvents::CloseRMenu)),
            )
            .unwrap();
        Self {
            visible: false,
            _producer: RMenuBus::bridge(Rc::new(cb)),
            pos_x: 0,
            pos_y: 0,
            close_events: Vec::new(),
            _location_handle: listener,
            content: Html::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let clear_events = || {
            if !self.close_events.is_empty() {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let body = document.body().unwrap();

                for close_event in self.close_events.iter() {
                    for event in JS_EVENTS_TO_CANCEL {
                        body.remove_event_listener_with_callback(event, close_event)
                            .unwrap();
                    }
                }
            }
        };
        match msg {
            Msg::HandleBusMessage(m) => match m {
                RMenusBusEvents::OpenRMenu(x, y, kind) => {
                    clear_events();
                    self.visible = true;
                    self.pos_x = x;
                    self.pos_y = y;

                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    let body = document.body().unwrap();

                    let close_event = close_context_menu_function();

                    for event in JS_EVENTS_TO_CANCEL {
                        body.add_event_listener_with_callback(event, &close_event)
                            .unwrap();
                    }

                    self.content = match kind {
                        RMenuKind::MessageRMenu(props) => html! { <MessageRMenu ..props />},
                        RMenuKind::ProfileRMenu(props) => html! { <ProfileRMenu ..props /> },
                    };

                    self.close_events.push(close_event);
                }
                RMenusBusEvents::CloseRMenu => {
                    clear_events();
                    self.content = Html::default();
                    self.visible = false;
                }
            },
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let style = format!("left: {}px; top : {}px;", self.pos_x, self.pos_y);
        html! {
            <div id="rmenu" hidden={!self.visible} {style} class={"z-1000 absolute dark:bg-zinc-600 dark:text-white border-10 p-2 rounded-md border-zinc-500 whitespace-nowrap"}>
                {self.content.clone()}
            </div>
        }
    }
}
